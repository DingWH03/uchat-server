// src/client.rs
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use anyhow::Result;
use sqlx::MySqlPool;
use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{read_packet, send_packet};
use crate::handlers::handle_authenticated_client;
use bcrypt::{hash, verify};

#[derive(Clone)]
pub struct Client {
    pub socket: Arc<Mutex<TcpStream>>,
    pub username: Option<String>,
    pub user_id: Option<String>,
}

impl Client {
    /// 创建一个新的客户端实例，并将其包装在 `Arc<Mutex<>>` 中以实现线程安全的共享。
    pub fn new(socket: TcpStream) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Client {
            socket: Arc::new(Mutex::new(socket)),
            username: None,
            user_id: None,
        }))
    }

    /// 处理客户端的注册和登录请求。
    pub async fn handle(
        client_arc: Arc<Mutex<Client>>,
        pool: MySqlPool,
        clients: Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>,
    ) -> Result<()> {
        // 锁定客户端以进行操作
        let mut client = client_arc.lock().await;

        // 读取客户端的请求包
        let request_json = match read_packet(&mut *client.socket.lock().await).await {
            Ok(json) => json,
            Err(_) => {
                // 无法读取请求，关闭连接
                return Ok(());
            }
        };

        // 解析客户端请求
        let request: ClientRequest = match serde_json::from_value(request_json) {
            Ok(req) => req,
            Err(_) => {
                let response = ServerResponse::Error {
                    message: "无效的请求格式".to_string(),
                };
                send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?) .await?;
                return Ok(());
            }
        };

        match request {
            ClientRequest::Register { username, password } => {
                // 处理注册请求
                let hashed = hash(&password, 4)?;
                let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
                    .bind(&username)
                    .bind(&hashed)
                    .execute(&pool).await;

                match res {
                    Ok(_) => {
                        let response = ServerResponse::AuthResponse {
                            status: "ok".to_string(),
                            message: "注册成功".to_string(),
                        };
                        send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;
                    },
                    Err(e) => {
                        let response = ServerResponse::Error {
                            message: format!("注册失败: {}", e),
                        };
                        send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;
                        return Ok(());
                    }
                }
                return Ok(());
            },
            ClientRequest::Login { username, password } => {
                // 处理登录请求
                let row = sqlx::query!("SELECT id, password_hash FROM users WHERE username = ?", username)
                    .fetch_optional(&pool).await?;

                if let Some(user) = row {
                    if verify(&password, &user.password_hash)? {
                        client.user_id = Some(user.id.to_string());
                        client.username = Some(username.clone());

                        let response = ServerResponse::AuthResponse {
                            status: "success".to_string(),
                            message: "登录成功".to_string(),
                        };
                        send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;

                        // 将用户加入在线列表
                        if let Some(ref user_id) = client.user_id {
                            clients.lock().await.insert(user_id.clone(), Arc::clone(&client_arc));
                            println!("用户 {} 已登录", username);
                        }

                        // 解锁客户端并开始处理后续消息
                        drop(client);
                        handle_authenticated_client(client_arc, pool, clients).await?;
                    } else {
                        let response = ServerResponse::AuthResponse {
                            status: "error".to_string(),
                            message: "密码错误".to_string(),
                        };
                        send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;
                        return Ok(());
                    }
                } else {
                    let response = ServerResponse::AuthResponse {
                        status: "error".to_string(),
                        message: "用户不存在".to_string(),
                    };
                    send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;
                    return Ok(());
                }
            },
            ClientRequest::SendMessage { .. } => {
                // 未登录时尝试发送消息
                let response = ServerResponse::Error {
                    message: "请先登录".to_string(),
                };
                send_packet(&mut *client.socket.lock().await, &serde_json::to_value(response)?).await?;
                return Ok(());
            },
        }

        Ok(())
    }
}

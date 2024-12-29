// src/handlers.rs
use crate::client::Client;
use sqlx::MySqlPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use anyhow::Result;
use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{read_packet, send_packet};
use serde_json::Value;
use chrono::Utc;

pub async fn handle_authenticated_client(
    client_arc: Arc<Mutex<Client>>,
    pool: MySqlPool,
    clients: Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>>,
) -> Result<()> {
    loop {
        // 读取客户端发送的消息包
        let msg_json = match read_packet(&mut *client_arc.lock().await.socket.lock().await).await {
            Ok(json) => json,
            Err(_) => {
                // 连接关闭
                let client = client_arc.lock().await;
                if let Some(ref user_id) = client.user_id {
                    clients.lock().await.remove(user_id);
                    if let Some(ref username) = client.username {
                        println!("用户 {} 已断开连接", username);
                    }
                }
                break;
            }
        };
        println!("{:?}", msg_json);

        // 解析客户端消息
        let msg_request: ClientRequest = match serde_json::from_value(msg_json) {
            Ok(req) => req,
            Err(_) => {
                let response = ServerResponse::Error {
                    message: "无效的请求格式".to_string(),
                };
                send_packet(&mut *client_arc.lock().await.socket.lock().await, &serde_json::to_value(response)?).await?;
                continue;
            }
        };

        match msg_request {
            ClientRequest::SendMessage { receiver, message } => {
                // 处理发送消息请求
                let client = client_arc.lock().await;
                let user_id = client.user_id.clone().unwrap();
                let username = client.username.clone().unwrap();
                drop(client); // 解锁以避免死锁

                println!("用户 {} 发送消息给 {}", username, receiver);
                
                // 查询接收者的ID
                let receiver_row = sqlx::query!("SELECT id FROM users WHERE username = ?", receiver)
                    .fetch_optional(&pool).await?;

                if let Some(receiver_user) = receiver_row {
                    let receiver_id = receiver_user.id.to_string();

                    // 将消息存储到数据库
                    sqlx::query("INSERT INTO messages (sender_id, receiver_id, message) VALUES (?, ?, ?)")
                        .bind(&user_id)
                        .bind(&receiver_id)
                        .bind(&message)
                        .execute(&pool).await?;

                    // 检查接收者是否在线
                    let clients_guard = clients.lock().await;
                    if let Some(receiver_client_arc) = clients_guard.get(&receiver_id) {
                        // 构建接收消息
                        let receive_msg = ServerResponse::ReceiveMessage {
                            sender: username.clone(),
                            message: message.clone(),
                            timestamp: Utc::now().to_rfc3339(),
                        };
                        // 发送消息给接收者
                        let receiver_client = receiver_client_arc.lock().await;
                        send_packet(&mut *receiver_client.socket.lock().await, &serde_json::to_value(receive_msg)?).await?;
                    }

                    // 向发送者确认消息已发送
                    let response = ServerResponse::AuthResponse {
                        status: "ok".to_string(),
                        message: "消息已发送".to_string(),
                    };
                    send_packet(&mut *client_arc.lock().await.socket.lock().await, &serde_json::to_value(response)?).await?;
                } else {
                    let response = ServerResponse::Error {
                        message: "接收者不存在".to_string(),
                    };
                    send_packet(&mut *client_arc.lock().await.socket.lock().await, &serde_json::to_value(response)?).await?;
                }
            },
            ClientRequest::Register { .. } | ClientRequest::Login { .. } => {
                // 已登录后再次尝试注册或登录
                let response = ServerResponse::Error {
                    message: "您已登录".to_string(),
                };
                send_packet(&mut *client_arc.lock().await.socket.lock().await, &serde_json::to_value(response)?).await?;
            },
        }
    }

    Ok(())
}

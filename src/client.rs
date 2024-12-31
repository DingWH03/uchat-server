// src/client.rs
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;
use anyhow::Result;
use crate::protocol::{ClientRequest, ServerResponse};
use crate::utils::{reader_packet, writer_packet};
use crate::api::Api;

#[derive(Clone)]
pub struct Client {
    api: Arc<Mutex<Api>>,
    user_id: String,
    writer: Arc<Mutex<tokio::io::BufWriter<tokio::net::tcp::OwnedWriteHalf>>>,
    reader: Arc<Mutex<tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>>>,
    signed_in: bool,
}

impl Client {
    pub fn new(
        socket: TcpStream,
        api: Arc<Mutex<Api>>,
    ) -> Self {
        let (reader, writer) = socket.into_split();
        Self {
            api,
            user_id: String::new(),
            writer: Arc::new(Mutex::new(tokio::io::BufWriter::new(writer))),
            reader: Arc::new(Mutex::new(tokio::io::BufReader::new(reader))),
            signed_in: false,
        }
    }
    pub fn user_id(&self) -> String {
        self.user_id.clone()
    }
    pub async fn send_packet(&mut self, msg: &ServerResponse) -> Result<()> {
        writer_packet(&mut self.writer, &msg).await
    }
    pub async fn recv_packet(&mut self) -> Result<ClientRequest> {
        reader_packet(&mut self.reader).await
    }
    async fn handle_register(&self, username: String, password: String) -> ServerResponse {
        let status = {
            let mut api = self.api.lock().await;
            api.register(&username, &password).await
        };
    
        match status {
            Ok(true) => ServerResponse::AuthResponse {
                status: "ok".to_string(),
                message: "注册成功".to_string(),
            },
            Ok(false) => ServerResponse::AuthResponse {
                status: "error".to_string(),
                message: "用户名已存在".to_string(),
            },
            Err(err) => {
                eprintln!("注册失败: {:?}", err);
                ServerResponse::AuthResponse {
                    status: "error".to_string(),
                    message: "注册失败，请稍后重试".to_string(),
                }
            }
        }
    }
    

    async fn handle_login(&mut self, username: String, password: String) -> ServerResponse {
        let status = {
            let mut api = self.api.lock().await;
            api.login(&username, &password, Arc::new(Mutex::new(self.clone()))).await
        };
    
        match status {
            Ok(true) => {
                // 登录成功，更新用户状态
                self.user_id = username.clone();
                self.signed_in = true;
    
                ServerResponse::AuthResponse {
                    status: "success".to_string(),
                    message: "登录成功".to_string(),
                }
            }
            Ok(false) => ServerResponse::AuthResponse {
                status: "error".to_string(),
                message: "用户名或密码错误".to_string(),
            },
            Err(err) => {
                eprintln!("登录失败: {:?}", err);
                ServerResponse::AuthResponse {
                    status: "error".to_string(),
                    message: "登录失败，请稍后重试".to_string(),
                }
            }
        }
    }
    

    async fn handle_send_message(
        &self,
        sender: String,
        receiver: String,
        message: String,
    ) -> ServerResponse {
        let status = {
            let mut api = self.api.lock().await;
            api.send_message(&sender, &receiver, &message).await
        };

        ServerResponse::AuthResponse {
            status: if status { "ok".to_string() } else { "error".to_string() },
            message: if status { "消息发送成功".to_string() } else { "用户不存在".to_string() },
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = match self.recv_packet().await {
                Ok(req) => req,
                Err(e) => {
                    // 检测到连接断开
                    eprintln!("客户端连接断开，错误: {:?}", e);
                    // 调用 Api.down 方法处理账号下线逻辑
                    let mut api = self.api.lock().await;
                    api.down(&self.user_id).await;
                    break; // 跳出循环，停止处理客户端
                }
            };

            let response = match request {
                ClientRequest::Register { username, password } => {
                    self.handle_register(username, password).await
                }
                ClientRequest::Login { username, password } => {
                    self.handle_login(username, password).await
                }
                ClientRequest::SendMessage { receiver, message } => {
                    if !self.signed_in {
                        ServerResponse::Error {
                            message: "请先登录".to_string(),
                        }
                    } else {
                        self.handle_send_message(self.user_id.clone(), receiver, message).await
                    }
                }
            };

            // 尝试发送响应
            if let Err(e) = self.send_packet(&response).await {
                // 检测到发送失败（例如连接断开）
                eprintln!("发送数据失败，连接可能断开: {:?}", e);

                // 调用 Api.down 方法处理账号下线逻辑
                let mut api = self.api.lock().await;
                api.down(&self.user_id).await;

                break; // 跳出循环，停止处理客户端
            }
        }

        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("客户端对象销毁: {}", self.user_id);
        // 这里可以执行更多清理逻辑，例如从全局状态中移除客户端
    }
}

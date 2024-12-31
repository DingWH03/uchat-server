// src/main.rs
mod db;
mod protocol;
mod api;
mod utils;
mod client;

use api::Api;
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use anyhow::Result;
use db::init_db_pool;
use crate::client::Client;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化数据库连接池和表结构
    let pool = init_db_pool().await?;

    // 启动TCP服务器
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("服务器已启动，监听端口 8080");

    // 共享状态，存储已登录的用户
    // HashMap 的键为 user_id，值为 Arc<Mutex<Client>>
    let clients: HashMap<String, Arc<Mutex<Client>>> = HashMap::new();

    let api: Arc<Mutex<Api>> = Arc::new(Mutex::new(Api::new(pool,clients)));

    loop {
        if let Ok((socket, _)) = listener.accept().await {
            let api_clone = Arc::clone(&api);

            // 处理每个客户端连接
            tokio::spawn(async move {
                let mut client = Client::new(socket, api_clone);
                if let Err(e) = client.run().await {
                    eprintln!("客户端断开连接: {:?}", e);
                }
            });
            
        }
    }
}

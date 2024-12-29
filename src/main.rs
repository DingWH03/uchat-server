// src/main.rs
mod db;
mod models;
mod protocol;
mod handlers;
mod utils;
mod client; // 新增模块

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
    let clients: Arc<Mutex<HashMap<String, Arc<Mutex<Client>>>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("新连接来自: {}", addr);

        let pool = pool.clone();
        let clients = clients.clone();

        tokio::spawn(async move {
            // 创建一个新的客户端实例
            let client_arc = Client::new(socket);
            // 处理客户端请求
            if let Err(e) = Client::handle(client_arc.clone(), pool, clients).await {
                println!("处理客户端时出错: {:?}", e);
            }
        });
    }
}

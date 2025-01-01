use crate::client::{Client};
use bcrypt::hash;
use bcrypt::BcryptError;
use sqlx::mysql::MySqlPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex; // 引入 bcrypt 库

pub struct Api {
    pool: MySqlPool,
    clients: HashMap<String, Arc<Mutex<Client>>>,
}

impl Api {
    pub fn new(
        pool: sqlx::Pool<sqlx::MySql>,
        clients: HashMap<String, Arc<Mutex<Client>>>,
    ) -> Self {
        Self { pool, clients }
    }
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
        client: Arc<Mutex<Client>>, // 将自己的引用传进来
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, password_hash FROM users WHERE username = ?",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        // 如果用户不存在，直接返回 false
        let Some(row) = row else {
            return Ok(false);
        };

        // 从查询结果中提取密码哈希
        let password_hash = row.password_hash;

        // 验证用户输入的密码是否与数据库中的哈希值匹配
        match bcrypt::verify(password, &password_hash) {
            Ok(valid) => {
                if valid {
                    if let Some(client) = self.clients.get(username) {
                        let mut client = client.lock().await;
                        client.send_error("用户重复登录").await;
                        return Ok(false);
                    } else {
                        // 将客户端引用存入 clients 中
                        // &self.clients.insert(row.id.to_string(), client);
                        let _ = &self.clients.insert(username.to_string(), client);
                        return Ok(true);
                    }
                }
                Ok(valid)
            }
            Err(_) => Ok(false), // 如果验证失败或发生错误，返回 false
        }
    }
    pub async fn register(&self, username: &str, password: &str) -> Result<bool, BcryptError> {
        let hashed = hash(&password, 4)?;
        let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(&username)
            .bind(&hashed)
            .execute(&self.pool)
            .await;
        match res {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    pub async fn down(&self, user_id: &str) {
        println!("用户 {} 下线", user_id);
    }
    pub async fn send_message(&self, sender: &str, receiver: &str, message: &str) -> bool {
        if receiver == "all" {
            // 创建一个异步任务集合
            let clients = self.clients.clone(); // 克隆 Arc 引用

            // 创建一个异步任务集合
            let tasks: Vec<_> = clients
                .iter()
                .map(|(client_id, client)| {
                    let sender = sender.to_string();
                    let message = message.to_string();
                    let _client_id = client_id.clone(); // 克隆 client_id
                    let client = client.clone(); // 克隆锁

                    tokio::spawn(async move {
                        let mut client = client.lock().await;
                        client.receive_message(sender, message).await;
                    })
                })
                .collect();

            // 等待所有任务完成
            for task in tasks {
                if let Err(err) = task.await {
                    println!("并发任务失败: {}", err);
                }
            }
            true
        } else {
            // 单个接收者处理逻辑
            if let Some(client) = self.clients.get(receiver) {
                let mut client = client.lock().await;
                client
                    .receive_message(sender.to_string(), message.to_string())
                    .await;
                true
            } else {
                println!("接收者 {} 不在线或不存在", receiver);
                false
            }
        }
    }

    pub async fn online_users(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
}

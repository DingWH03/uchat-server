// src/db.rs
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use anyhow::Result;
use dotenv::dotenv;
use std::env;

pub async fn init_db_pool() -> Result<MySqlPool> {
    // 加载环境变量
    dotenv().ok();

    // 从环境变量读取数据库连接字符串
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL 环境变量未设置");

    // 连接到 MySQL 数据库
    // 连接到 MySQL 数据库，设置连接池大小
    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&database_url).await?;

    // 创建用户表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            username VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 创建消息表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id BIGINT PRIMARY KEY AUTO_INCREMENT,
            sender_id BIGINT,
            receiver_id BIGINT,
            message TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (sender_id) REFERENCES users(id),
            FOREIGN KEY (receiver_id) REFERENCES users(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

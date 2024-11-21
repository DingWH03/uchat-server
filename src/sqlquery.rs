use sqlx::{mysql::MySqlPoolOptions, Row};
use tokio;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 数据库连接字符串
    let database_url = "mysql://username:password@localhost:3306/my_database";

    // 创建连接池
    let pool = MySqlPoolOptions::new()
        .max_connections(5) // 设置最大连接数
        .connect(database_url)
        .await?;

    println!("成功连接到 MySQL 数据库");

    // 查询数据
    let rows = sqlx::query("SELECT id, name FROM users")
        .fetch_all(&pool)
        .await?;

    for row in rows {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        println!("用户ID: {}, 用户名: {}", id, name);
    }

    // 插入数据
    let result = sqlx::query("INSERT INTO users (name) VALUES (?)")
        .bind("新用户")
        .execute(&pool)
        .await?;

    println!("插入了 {} 行", result.rows_affected());

    Ok(())
}

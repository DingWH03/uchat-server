// src/utils.rs
use tokio::net::TcpStream;
use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const LENGTH_PREFIX_SIZE: usize = 4;

// 读取一个完整的数据包
pub async fn read_packet(stream: &mut TcpStream) -> Result<Value> {
    let mut len_buf = [0u8; LENGTH_PREFIX_SIZE];
    stream.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    stream.read_exact(&mut msg_buf).await?;

    let msg: Value = serde_json::from_slice(&msg_buf)?;
    // println!("收到消息: {:?}", msg);                        // 调试信息
    Ok(msg)
}

// 发送一个数据包
pub async fn send_packet(stream: &mut TcpStream, msg: &Value) -> Result<()> {
    let msg_str = serde_json::to_string(msg)?;
    let msg_bytes = msg_str.as_bytes();
    let msg_len = msg_bytes.len() as u32;
    let len_bytes = msg_len.to_be_bytes();

    stream.write_all(&len_bytes).await?;
    stream.write_all(msg_bytes).await?;
    Ok(())
}

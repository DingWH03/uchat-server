use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    // 服务端地址与端口
    let address = &env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS 未设置");

    // 启动监听器
    let listener = TcpListener::bind(address).expect("无法绑定到指定端口");
    println!("服务端已启动，正在监听 {}", address);

    // 用于存储所有客户端连接
    let clients = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 为每个客户端生成一个唯一标识符
                let client_id = stream.peer_addr().unwrap().to_string();
                println!("客户端已连接: {}", client_id);

                // 克隆共享的客户端列表
                let clients = Arc::clone(&clients);

                // 在新线程中处理该客户端
                thread::spawn(move || handle_client(stream, client_id, clients));
            }
            Err(e) => {
                eprintln!("接受连接时出错: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, client_id: String, clients: Arc<Mutex<HashMap<String, TcpStream>>>) {
    // 添加到共享的客户端列表
    {
        let mut clients_lock = clients.lock().unwrap();
        clients_lock.insert(client_id.clone(), stream.try_clone().expect("无法克隆流"));
    }

    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                // 客户端断开连接
                println!("客户端已断开: {}", client_id);
                break;
            }
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("收到来自 {} 的消息: {}", client_id, message);

                // 广播消息
                broadcast_message(&client_id, &message, &clients);
            }
            Err(e) => {
                eprintln!("读取数据时出错: {}", e);
                break;
            }
        }
    }

    // 从客户端列表中移除断开的客户端
    clients.lock().unwrap().remove(&client_id);
}

fn broadcast_message(sender_id: &str, message: &str, clients: &Arc<Mutex<HashMap<String, TcpStream>>>) {
    let clients_lock = clients.lock().unwrap();
    for (id, mut client) in clients_lock.iter() {
        if id != sender_id {
            let full_message = format!("{}: {}", sender_id, message);
            if let Err(e) = client.write_all(full_message.as_bytes()) {
                eprintln!("向客户端 {} 发送消息时出错: {}", id, e);
            }
        }
    }
}

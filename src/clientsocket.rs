use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::net::TcpListener;
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
}

struct ClientSocket {
    client_id: String,
    status: ConnectionStatus,
    cookie: Option<String>,
    last_connected: SystemTime,
}

impl ClientSocket {
    fn new(cookie: Option<String>) -> Self {
        ClientSocket {
            client_id: Uuid::new_v4().to_string(),
            status: ConnectionStatus::Connected,
            cookie,
            last_connected: SystemTime::now(),
        }
    }

    fn is_cookie_valid(&self) -> bool {
        match self.cookie {
            Some(ref cookie) => {
                // Check if cookie exists and is not expired
                let expiration_time = self.last_connected + Duration::new(7 * 24 * 3600, 0); // 7 days
                SystemTime::now() < expiration_time
            }
            None => false, // No cookie means invalid
        }
    }

    fn update_last_connected(&mut self) {
        self.last_connected = SystemTime::now();
    }

    fn renew_cookie(&mut self) {
        self.cookie = Some(Uuid::new_v4().to_string());
        self.update_last_connected();
    }
}

#[derive(Default)]
struct ClientManager {
    clients: HashMap<String, Arc<Mutex<ClientSocket>>>,
}

impl ClientManager {
    fn new() -> Self {
        ClientManager {
            clients: HashMap::new(),
        }
    }

    fn get_or_create_client(&mut self, cookie: Option<String>) -> Arc<Mutex<ClientSocket>> {
        if let Some(ref c) = cookie {
            // Try to find the existing client by cookie
            if let Some(existing_client) = self.clients.get(c) {
                let mut client = existing_client.lock().unwrap();
                if client.is_cookie_valid() {
                    client.update_last_connected();
                    return existing_client.clone();
                }
            }
        }
        
        // If no valid client, create a new one
        let new_client = Arc::new(Mutex::new(ClientSocket::new(cookie)));
        if let Some(ref c) = new_client.lock().unwrap().cookie {
            self.clients.insert(c.clone(), new_client.clone());
        }
        new_client
    }

    fn handle_client(&mut self, cookie: Option<String>) {
        let client_socket = self.get_or_create_client(cookie);

        let mut client = client_socket.lock().unwrap();
        match client.status {
            ConnectionStatus::Connected => {
                println!("Client {} is already connected.", client.client_id);
            }
            ConnectionStatus::Disconnected => {
                println!("Client {} is disconnected. Attempting to reconnect.", client.client_id);
                client.status = ConnectionStatus::Reconnecting;
            }
            ConnectionStatus::Reconnecting => {
                println!("Client {} is reconnecting.", client.client_id);
            }
        }
    }
}

fn main() {
    let client_manager = Arc::new(Mutex::new(ClientManager::new()));

    // Simulate a TCP listener accepting client connections
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(_) => {
                let client_manager = Arc::clone(&client_manager);
                thread::spawn(move || {
                    let cookie: Option<String> = Some(Uuid::new_v4().to_string()); // Mock cookie from client (e.g., HTTP header)

                    // Handle client connection (with or without cookie)
                    client_manager.lock().unwrap().handle_client(cookie);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

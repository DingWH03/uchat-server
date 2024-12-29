// src/protocol.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ClientRequest {
    #[serde(rename = "register")]
    Register {
        username: String,
        password: String,
    },
    #[serde(rename = "login")]
    Login {
        username: String,
        password: String,
    },
    #[serde(rename = "send_message")]
    SendMessage {
        receiver: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ServerResponse {
    #[serde(rename = "auth_response")]
    AuthResponse {
        status: String,
        message: String,
    },
    #[serde(rename = "receive_message")]
    ReceiveMessage {
        sender: String,
        message: String,
        timestamp: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

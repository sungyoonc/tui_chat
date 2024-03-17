use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageKind {
    Chat {
        id: u64,
        username: String,
        msg: String,
    },
}

#[derive(Debug)]
pub struct Connection {
    pub id: u64,
    pub username: String,
    pub current_channel: u64,
    pub sender: mpsc::UnboundedSender<Message>,
}

impl Connection {
    pub fn new(
        id: u64,
        username: String,
        current_channel: u64,
        sender: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            id,
            username,
            current_channel,
            sender,
        }
    }

    pub fn send(&self, channel: u64, message: &MessageKind) -> bool {
        if self.current_channel != channel {
            return false;
        }

        let data: String = match serde_json::to_string(message) {
            Ok(data) => data,
            Err(_) => {
                return false;
            }
        };
        self.sender.send(Message::text(data)).is_ok()
    }
}

pub type Connections = Arc<RwLock<HashMap<u64, Connection>>>; // id, User

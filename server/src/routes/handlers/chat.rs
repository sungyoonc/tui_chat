use crate::db::Database;
use crate::models::chat::{Connection, Connections, MessageKind};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

pub async fn ws(
    websocket: WebSocket,
    connections: Connections,
    database: Database,
    id: u64,
    channel: u64,
) {
    let (mut ws_tx, mut ws_rx) = websocket.split();
    let (tx, rx) = mpsc::unbounded_channel::<Message>();

    // When receive message send it to the user
    tokio::task::spawn(async move {
        let mut rx = UnboundedReceiverStream::new(rx);
        while let Some(message) = rx.next().await {
            ws_tx.send(message).await.unwrap_or(());
        }
    });

    let username = match database.get_username(id).await {
        Some(username) => username,
        None => {
            return;
        }
    };

    connections
        .write()
        .await
        .insert(id, Connection::new(id, username.clone(), channel, tx));

    while let Some(res) = ws_rx.next().await {
        let msg = match res {
            Ok(msg) => msg,
            Err(_) => {
                break;
            }
        };
        if msg.is_close() {
            break;
        }
        if msg.is_text() {
            let new_msg = MessageKind::Chat {
                id,
                username: username.clone(),
                msg: msg.to_str().unwrap().to_owned(),
            };
            for (&from_id, connection) in connections.read().await.iter() {
                if id == from_id {
                    continue;
                }
                connection.send(channel, &new_msg);
            }
        }
    }
}

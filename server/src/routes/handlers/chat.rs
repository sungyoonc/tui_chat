use std::time::SystemTime;

use crate::db::Database;
use crate::models::chat::{ChatTokenInfo, ChatTokenResponse, MessageKind};
use crate::models::chat::{Connection, Connections};
use crate::routes::AuthDetail;
use crate::utils;

use futures_util::{SinkExt, StreamExt};
use mysql::prelude::Queryable;
use mysql::{params, Row};
use rand_core::{OsRng, RngCore};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

const CHAT_TOKEN_EXPIRE_MINUTE: u64 = 5;

pub async fn chat_token(
    auth: AuthDetail,
    database: Database,
    channel: u64,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: add check if channel is valid
    let mut conn = database.pool.get_conn().unwrap();

    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut chat_token_source = auth.id.to_le_bytes().to_vec();
    chat_token_source.append(&mut key);
    let chat_token = utils::hash_from_u8(chat_token_source);

    let expire = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60 * CHAT_TOKEN_EXPIRE_MINUTE;

    conn.exec::<Row, _, _>(
        r"
            INESRT INTO chat_token (chat_token, expire, channel, session, is_used)
            VALUES(:chat_token, :expire, :channel, :session: :is_used)",
        params! {
            "chat_token" => chat_token.clone(),
            "expire" => expire,
            "channel" => channel,
            "session" => auth.session,
            "is_used" => false,
        },
    )
    .unwrap();

    let response = ChatTokenResponse { chat_token };
    Ok(warp::reply::json(&response))
}

pub async fn ws(
    websocket: WebSocket,
    connections: Connections,
    database: Database,
    token_info: ChatTokenInfo,
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

    let username = match database.get_username(token_info.id).await {
        Some(username) => username,
        None => {
            return;
        }
    };

    connections.write().await.insert(
        token_info.token.clone(),
        Connection::new(token_info.id, username.clone(), token_info.channel, tx),
    );

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
                id: token_info.id,
                username: username.clone(),
                msg: msg.to_str().unwrap().to_owned(),
            };
            for (from_id, connection) in connections.read().await.iter() {
                if token_info.id.to_string() == *from_id {
                    continue;
                }
                connection.send(token_info.channel, &new_msg);
            }
        }
    }

    // Cleanup
    connections.write().await.remove(&token_info.token);
    let mut conn = database.pool.get_conn().unwrap();
    conn.exec::<Row, _, _>(
        "DELETE FROM chat_token WHERE chat_token = :chat_token",
        params! {"chat_token" => token_info.token},
    )
    .unwrap();
}

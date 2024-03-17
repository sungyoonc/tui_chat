use mysql::{params, prelude::Queryable, Pool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::configuration::DatabaseSettings;
use crate::models::chat::ChatTokenInfo;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub fn new(settings: &DatabaseSettings) -> Self {
        let id = &settings.username;
        let pw = &settings.password;
        let hostname = &settings.hostname;
        let port = settings.port;
        let name = &settings.name;

        Self {
            pool: Pool::new(
                format!("mysql://{}:{}@{}:{}/{}", id, pw, hostname, port, name).as_str(),
            )
            .expect("Can't connect to mysql server"),
        }
    }
    pub fn db_setup(&self) {
        let mut conn = self.pool.get_conn().unwrap();
        conn.exec::<Vec<_>, &str, ()>(
            "
        CREATE TABLE IF NOT EXISTS login (
        id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
        username VARCHAR(32) UNIQUE KEY,
        pw VARCHAR(64),
        salt VARCHAR(64));",
            (),
        )
        .unwrap();
        conn.exec::<Vec<_>, &str, ()>(
            "
        CREATE TABLE IF NOT EXISTS session (
        session VARCHAR(64) PRIMARY KEY,
        id BIGINT UNSIGNED,
        is_remember BOOLEAN,
        expire BIGINT UNSIGNED,
        refresh_token VARCHAR(64) UNIQUE KEY,
        refresh_expire BIGINT UNSIGNED);",
            (),
        )
        .unwrap();
        conn.exec::<Vec<_>, &str, ()>(
            "
        CREATE TABLE IF NOT EXISTS chat_token (
        chat_token VARCHAR(64) PRIMARY KEY,
        expire BIGINT UNSIGNED,
        channel VARCHAR(64),
        session VARCHAR(64),
        is_used BOOLEAN);",
            (),
        )
        .unwrap();
    }

    pub async fn check_session(&self, session: String) -> Option<u64> {
        let mut conn = self.pool.get_conn().unwrap();
        let result: Vec<Row> = conn
            .exec(
                "SELECT id, expire, refresh_expire FROM session WHERE session = :session",
                params! {"session" => session.clone()},
            )
            .unwrap();

        // check if session exists
        if result.is_empty() {
            return None;
        }

        // check if session is vaild
        let (id, expire, refresh_expire): (u64, u64, u64) = mysql::from_row(result[0].clone());
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if current_time > expire {
            if current_time > refresh_expire {
                // delete expired session
                conn.exec::<Row, _, _>(
                    "DELETE FROM session WHERE session = :session",
                    params! {"session" => session.clone()},
                )
                .unwrap();
                // delete associated chat_tokens
                conn.exec::<Row, _, _>(
                    "DELETE FROM chat_token WHERE session = :session",
                    params! {"session" => session},
                )
                .unwrap();
            }
            return None;
        }

        return Some(id);
    }

    pub async fn get_username(&self, id: u64) -> Option<String> {
        let mut conn = self.pool.get_conn().unwrap();
        let result: Vec<Row> = conn
            .exec(
                r"SELECT username FROM login WHERE id = :id",
                params! {"id" => id},
            )
            .unwrap();

        if result.is_empty() {
            return None;
        }

        let username: String = mysql::from_row(result[0].clone());

        Some(username)
    }

    pub async fn check_chat_token(&self, chat_token: String) -> Option<ChatTokenInfo> {
        let mut conn = self.pool.get_conn().unwrap();
        let result: Vec<Row> = conn
            .exec(
                r"
                SELECT
                  s.id, l.username, t.is_used, t.channel, s.expire, t.expire
                FROM chat_token t
                JOIN session s
                  ON t.session = s.session
                    AND t.chat_token = :chat_token
                JOIN login l
                  ON s.id = l.id;",
                params! {"chat_token" => chat_token.clone()},
            )
            .unwrap();
        if result.is_empty() {
            return None;
        }

        let (id, username, is_used, channel, session_expire, chat_token_expire): (
            u64,
            String,
            bool,
            u64,
            u64,
            u64,
        ) = mysql::from_row(result[0].clone());

        if is_used {
            return None;
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if current_time > session_expire {
            return None;
        }

        if current_time > chat_token_expire {
            conn.exec::<Row, _, _>(
                r"DELETE FROM chat_token WHERE expire < :current_time",
                params! {"current_time" => current_time},
            )
            .unwrap();
            return None;
        }

        conn.exec::<Row, _, _>(
            "UPDATE chat_token SET is_used=TRUE WHERE chat_token=:chat_token",
            params! {"chat_token" => chat_token.clone()},
        )
        .unwrap();

        Some(ChatTokenInfo {
            token: chat_token,
            id,
            username,
            channel,
        })
    }
}

use mysql::{params, prelude::Queryable, Pool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::configuration::DatabaseSettings;

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
            // delete expired session
            if current_time > refresh_expire {
                conn.exec::<Row, _, _>(
                    "DELETE FROM session WHERE session = :session",
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
}

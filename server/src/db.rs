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
        id BIGINT UNSIGNED PRIMARY KEY,
        username VARCHAR(32) UNIQUE KEY,
        pw VARCHAR(64),
        salt VARCHAR(64),
        refresh_token VARCHAR(64));",
            (),
        )
        .unwrap();
        conn.exec::<Vec<_>, &str, ()>(
            "
        CREATE TABLE IF NOT EXISTS session (
        id BIGINT UNSIGNED,
        session VARCHAR(64),
        expire BIGINT UNSIGNED)",
            (),
        )
        .unwrap();
    }
    pub fn check_session(&self, session: String) -> Option<u64> {
        let mut conn = self.pool.get_conn().unwrap();
        let result: Vec<Row> = conn
            .exec(
                "SELECT id, expire FROM session WHERE session = :session",
                params! {"session" => session.clone()},
            )
            .unwrap();

        // check if session exists
        if result.len() == 0 {
            return None;
        }

        // check if session is vaild
        let (id, expire): (u64, u64) = mysql::from_row(result[0].clone());
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if current_time > expire {
            // delete expired session
            let _result: Vec<Row> = conn
                .exec(
                    "DELETE FROM session WHERE session = :session",
                    params! {"session" => session},
                )
                .unwrap();
            return None;
        }

        return Some(id);
    }
}

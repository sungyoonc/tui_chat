use mysql::{prelude::Queryable, Pool, Row, params};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Db {
    pub pool: Pool
}

impl Db {
    pub fn new() -> Db{
        let id = env::var("MYSQL_ID").expect("No 'MYSQL_ID' env variable");
        let pw = env::var("MYSQL_PW").expect("No 'MYSQL_PW' env variable");
        let db_name = env::var("MYSQL_DB_NAME").expect("No 'MYSQL_DB_NAME' env variable");
        let db_port = env::var("MYSQL_PORT").unwrap_or("3306".to_string());
        Db {pool: Pool::new(format!("mysql://{}:{}@localhost:{}/{}", id, pw, db_port, db_name).as_str()).expect("Can't connect to mysql server")}
    }
}

pub fn db_setup() {
    let db = Db::new();
    let mut conn = db.pool.get_conn().unwrap();
    conn.exec::<Vec<_>, &str, ()>("
        CREATE TABLE IF NOT EXISTS login (
        id BIGINT UNSIGNED PRIMARY KEY,
        username VARCHAR(32) UNIQUE KEY,
        pw VARCHAR(64),
        salt VARCHAR(64),
        refresh_token VARCHAR(64));", ()).unwrap();
    conn.exec::<Vec<_>, &str, ()>("
        CREATE TABLE IF NOT EXISTS session (
        id BIGINT UNSIGNED,
        session VARCHAR(64),
        expire BIGINT UNSIGNED)", ()).unwrap();
}

pub fn check_session(session: String) -> Option<u64> {
    let mut conn = Db::new().pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec("SELECT id, expire FROM session WHERE session = :session", params! {"session" => session.clone()}).unwrap();
    
    // check if session exists
    if result.len() == 0 {
        return None
    }
    
    // check if session is vaild
    let (id, expire): (u64, u64) = mysql::from_row(result[0].clone());
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if current_time > expire {
        // delete expired session
        let mut conn = Db::new().pool.get_conn().unwrap();
        let _result: Vec<Row> = conn.exec("DELETE FROM session WHERE session = :session", params! {"session" => session}).unwrap();
        return None
    }

    return Some(id);
}
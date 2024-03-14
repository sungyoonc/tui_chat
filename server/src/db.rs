use mysql::{prelude::Queryable, Pool, Row, params};
use std::env;

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
        id VARCHAR(64),
        pw VARCHAR(64),
        salt VARCHAR(64),
        );", ()).unwrap();
    conn.exec::<Vec<_>, &str, ()>("
        CREATE TABLE IF NOT EXISTS session (
        id VARCHAR(64),
        session VARCHAR(64),
        expire VARCHAR(64),", ()).unwrap();
}

pub fn check_session(session: String) -> Option<String> {
    let mut conn = Db::new().pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec("SELECT id FROM session WHERE session = :session", params! {"session" => session}).unwrap();
    if result.len() == 0 {
        return None
    }
    mysql::from_row(result[0].clone())
}
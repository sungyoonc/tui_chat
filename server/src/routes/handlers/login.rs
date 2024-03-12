use std::convert::Infallible;
use crate::routes::*;
use mysql::{params, prelude::Queryable, Pool, Row};
use std::env;
use crate::utils;
use rand_core::{RngCore, OsRng};
use chrono::{prelude::*, TimeDelta};
use warp::http::StatusCode;

static SESSION_EXPIRE_HOUR: i64 = 1;

struct Db {
    pool: Pool
}

impl Db {
    fn new() -> Db{
        let id = env::var("MYSQL_ID").expect("No 'MYSQL_ID' env variable");
        let pw = env::var("MYSQL_PW").expect("No 'MYSQL_PW' env variable");
        let db_name = env::var("MYSQL_DB_NAME").expect("No 'MYSQL_DB_NAME' env variable");
        let db_port = env::var("MYSQL_PORT").unwrap_or("3306".to_string());
        Db {pool: Pool::new(format!("mysql://{}:{}@localhost:{}/{}", id, pw, db_port, db_name).as_str()).expect("Can't connect to mysql server")}
    }    
}


pub async fn login(json_data: LoginData) -> Result<impl warp::Reply, Infallible> {
    let hashed_id = utils::hash_from_string(json_data.clone().id);
    let hashed_pw = utils::hash_from_string(json_data.pw);

    let mut conn = Db::new().pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec("SELECT * FROM login WHERE id = :id AND pw= :pw", params! {"id" => hashed_id.clone(), "pw" => hashed_pw}).unwrap();

    if result.len() != 1 {
        return Ok(warp::reply::with_status("login error", StatusCode::UNAUTHORIZED))
    }

    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut session_source = hashed_id.clone().into_bytes();
    session_source.append(&mut key);
    let session = utils::hash_from_u8(session_source);
    let expire_time = Utc::now() + TimeDelta::try_hours(SESSION_EXPIRE_HOUR).unwrap();
    let expire = format!("{}-{}-{} {}:{}:{}.{}", expire_time.year(), expire_time.month(), expire_time.day(), expire_time.hour(), expire_time.minute(), expire_time.second(), (expire_time.nanosecond() as i64) % 10000000);
    let _result: Vec<Row> = conn.exec("INSERT INTO session (id, session, expire) VALUES (:id, :session, :expire)", params! {"id" => hashed_id, "session" => session.clone(), "expire" => expire}).unwrap();

    return Ok(warp::reply::with_status("login success", StatusCode::OK))
}
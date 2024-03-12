use std::convert::Infallible;
use crate::routes::*;
use lazy_static::lazy_static;
use mysql::{params, prelude::Queryable, Pool, Row};
use std::env;
use crate::utils;
use rand_core::{RngCore, OsRng};
use chrono::{prelude::*, TimeDelta};
use warp::http::StatusCode;

static SESSION_EXPIRE_HOUR: i64 = 1;
lazy_static! {
    static ref POOL: Pool = Pool::new(format!("mysql://{}:{}@localhost:3306/{}", env::var("MYSQL_ID").unwrap(), env::var("MYSQL_PW").unwrap(), env::var("MYSQL_DB_NAME").unwrap()).as_str()).unwrap();
}

pub async fn login(json_data: LoginData) -> Result<impl warp::Reply, Infallible> {
    let hashed_id = utils::hash_from_string(json_data.clone().id);
    let hashed_pw = utils::hash_from_string(json_data.pw);

    let mut conn = POOL.get_conn().unwrap();
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
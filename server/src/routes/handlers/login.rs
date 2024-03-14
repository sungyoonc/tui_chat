use std::convert::Infallible;
use crate::routes::*;
use mysql::{params, prelude::Queryable, Row};
use crate::utils;
use rand_core::{RngCore, OsRng};
use warp::http::StatusCode;
use crate::db::Db;
use std::time::{SystemTime, UNIX_EPOCH};

static SESSION_EXPIRE_HOUR: u64 = 1;

pub async fn login(json_data: LoginData) -> Result<impl warp::Reply, Infallible> {
    let id = json_data.clone().id;
    let pw = json_data.pw;

    let mut conn = Db::new().pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec("SELECT salt, pw FROM login WHERE id = :id", params! {"id" => id.clone()}).unwrap();
    if result.len() == 0 {
        return Ok(StatusCode::UNAUTHORIZED)
    }
    let (salt, db_pw): (String, String) = mysql::from_row(result[0].clone());

    let hashed_pw = utils::hash_from_string(format!("{}{}", pw, salt));
    if hashed_pw != db_pw {
        return Ok(StatusCode::UNAUTHORIZED)
    }

    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut session_source = id.clone().into_bytes();
    session_source.append(&mut key);
    let session = utils::hash_from_u8(session_source);
    
    let expire_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 60*60*SESSION_EXPIRE_HOUR;

    // let expire_time = Utc::now() + TimeDelta::try_hours(SESSION_EXPIRE_HOUR).unwrap();
    // let expire = format!("{}-{}-{} {}:{}:{}.{}", expire_time.year(), expire_time.month(), expire_time.day(), expire_time.hour(), expire_time.minute(), expire_time.second(), (expire_time.nanosecond() as i64) % 10000000);
    let _result: Vec<Row> = conn.exec("INSERT INTO session (id, session, expire) VALUES (:id, :session, :expire)", params! {"id" => id, "session" => session.clone(), "expire" => expire_time}).unwrap();

    return Ok(StatusCode::OK)
}
use crate::db::Database;
use crate::routes::ApiError;
use crate::routes::*;
use crate::utils;

use mysql::{params, prelude::Queryable, Row};
use rand_core::{OsRng, RngCore};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::reject::Rejection;

const SESSION_DEFAULT_EXPIRE_MINUTE: u64 = 30;
const REFRESH_REMEMBER_EXPIRE_HOUR: u64 = 24 * 7;
const REFRESH_NO_REMEMBER_EXPIRE_HOUR: u64 = 1;

// response format
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub struct ResponseData {
    session: String,
    refresh_token: String,
}

pub async fn login(
    json_data: LoginData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let username: String = json_data.clone().username;
    let pw = json_data.pw;

    // get salt and pw from login table
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn
        .exec(
            "SELECT id, salt, pw FROM login WHERE username = :username",
            params! {"username" => username.clone()},
        )
        .unwrap();
    if result.is_empty() {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // check if user pw is correct
    let (id, salt, db_pw): (u64, String, String) = mysql::from_row(result[0].clone());
    let hashed_pw = utils::hash_from_string(format!("{}{}", pw, salt));
    if hashed_pw != db_pw {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // cleanup expired sessions
    let result: Vec<Row> = conn
        .exec(
            "SELECT session, refresh_expire FROM session WHERE id = :id",
            params! {"id" => id},
        )
        .unwrap();
    if result.is_empty() {
        for row in result {
            let (session, refresh_expire): (String, u64) = mysql::from_row(row);
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if current_time > refresh_expire {
                // delete expired session
                let _result: Vec<Row> = conn
                    .exec(
                        "DELETE FROM session WHERE session = :session",
                        params! {"session" => session},
                    )
                    .unwrap();
            }
        }
    }

    // make session by hashing random number and id
    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut session_source = id.clone().to_string().into_bytes();
    session_source.append(&mut key);
    let session = utils::hash_from_u8(session_source);
    // make refresh_toke by hashing random number and id
    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut refresh_token_source = id.clone().to_string().into_bytes();
    refresh_token_source.append(&mut key);
    let refresh_token = utils::hash_from_u8(refresh_token_source);
    // make normal and refresh expire time
    let expire = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60 * SESSION_DEFAULT_EXPIRE_MINUTE;
    let refresh_expire = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + match json_data.remember {
            true => 60 * 60 * REFRESH_REMEMBER_EXPIRE_HOUR,
            false => 60 * 60 * REFRESH_NO_REMEMBER_EXPIRE_HOUR,
        };
    // insert session to the session table
    conn.exec::<Row, _, _>(
        "INSERT INTO session (session, id, is_remember, expire, refresh_token, refresh_expire)
        VALUES (:session, :id, :is_remember :expire, :refresh_token :refresh_expire)",
        params! {
            "id" => id,
            "session" => session.clone(),
            "is_remember" => json_data.remember,
            "expire" => expire,
            "refresh_token" => refresh_token.clone(),
            "refresh_expire" => refresh_expire
        },
    )
    .unwrap();

    // response
    let response = ResponseData {
        session,
        refresh_token,
    };

    return Ok(warp::reply::json(&response));
}

pub async fn refresh(
    json_data: RefreshData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    // check if the refresh token is valid
    let refresh_token = json_data.refresh_token;
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn
        .exec(
            "SELECT id, is_remember, refresh_expire FROM login WHERE refresh_token = :refresh_token",
            params! {"refresh_token" => refresh_token.clone()},
        )
        .unwrap();
    if result.is_empty() {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    let (id, is_remember, refresh_expire): (String, bool, u64) = mysql::from_row(result[0].clone());

    // Delete old session regardless of validity
    conn.exec::<Row, _, _>(
        "DELETE FROM session WHERE refresh_token = :refresh_token",
        params! {"refresh_token" => refresh_token},
    )
    .unwrap();

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if current_time > refresh_expire {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // make session by hashing random number and id
    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut session_source = id.clone().into_bytes();
    session_source.append(&mut key);
    let session = utils::hash_from_u8(session_source);
    // make refresh_toke by hashing random number and id
    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut refresh_token_source = id.clone().to_string().into_bytes();
    refresh_token_source.append(&mut key);
    let refresh_token = utils::hash_from_u8(refresh_token_source);
    // make normal and refresh expire time
    let expire = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60 * SESSION_DEFAULT_EXPIRE_MINUTE;
    let refresh_expire = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + match is_remember {
            true => 60 * 60 * REFRESH_REMEMBER_EXPIRE_HOUR,
            false => 60 * 60 * REFRESH_NO_REMEMBER_EXPIRE_HOUR,
        };
    // insert session to the session table
    conn.exec::<Row, _, _>(
        "INSERT INTO session (session, id, is_remember, expire, refresh_token, refresh_expire)
        VALUES (:session, :id, :is_remember :expire, :refresh_token :refresh_expire)",
        params! {
            "id" => id,
            "session" => session.clone(),
            "is_remember" => is_remember,
            "expire" => expire,
            "refresh_token" => refresh_token.clone(),
            "refresh_expire" => refresh_expire
        },
    )
    .unwrap();

    // reponse
    let response = ResponseData {
        session,
        refresh_token,
    };
    return Ok(warp::reply::json(&response));
}

pub async fn signup(
    json_data: SignupData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let username = json_data.clone().username;
    let pw = json_data.pw;

    // check if username is already in the database
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn
        .exec(
            "SELECT id FROM login WHERE username = :username",
            params! {"username" => username.clone()},
        )
        .unwrap();
    if !result.is_empty() {
        let invalid_params_vec: Vec<InvalidParamsDetail> = vec![InvalidParamsDetail {
            name: "username".to_string(),
            reason: "username already taken".to_string(),
        }];
        return Err(warp::reject::custom(ApiError::NotProcessable(
            invalid_params_vec,
        )));
    }

    // create salt and insert salt and pw to database
    let mut key = OsRng.next_u64().to_le_bytes().to_vec();
    let mut salt_source = username.clone().to_string().into_bytes();
    salt_source.append(&mut key);
    let salt = utils::hash_from_u8(salt_source);
    let hashed_pw = utils::hash_from_string(format!("{}{}", pw, salt));
    let _result: Vec<Row> = conn
        .exec(
            "INSERT INTO login (salt, pw, username) VALUES (:salt, :pw, :username)",
            params! {"salt" => salt, "pw" => hashed_pw, "username" => username},
        )
        .unwrap();

    Ok(warp::reply())
}

pub async fn logout(
    json_data: LogoutData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let session = json_data.clone().session;

    // delete session in the database
    let mut conn = database.pool.get_conn().unwrap();
    let _result: Vec<Row> = conn
        .exec(
            "DELETE FROM session WHERE session = :session",
            params! {"session" => session},
        )
        .unwrap();

    Ok(warp::reply())
}

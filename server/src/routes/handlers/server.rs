use crate::db::Database;
use crate::routes::*;

use block_id::{Alphabet, BlockId};
use mysql::{params, prelude::Queryable, Row};
use serde::Serialize;
use warp::reject::Rejection;

#[derive(Serialize)]
pub struct InviteCodeData {
    invite_code: String,
}

#[derive(Serialize)]
pub struct SearchData {
    id: u64,
    name: String,
    invite_code: String,
}

pub async fn join(
    auth: AuthDetail,
    json_data: ServerJoinData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let user_id = auth.id;
    let invite_code = json_data.invite_code;

    // check if invite code is in the database
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn
        .exec(
            "SELECT id FROM server WHERE invite_code = :invite_code",
            params! {
                "invite_code" => invite_code,
            },
        )
        .unwrap();

    if result.is_empty() {
        let invalid_params_vec: Vec<InvalidParamsDetail> = vec![InvalidParamsDetail {
            name: "invite_code".to_string(),
            reason: "No such invite_code".to_string(),
        }];
        return Err(warp::reject::custom(ApiError::NotProcessable(
            invalid_params_vec,
        )));
    }

    // add authority info to user_server_relationship table
    let server_id: u64 = mysql::from_row(result[0].clone());
    conn.exec::<Row, _, _>(
        "INSERT IGNORE INTO
        user_server_relationship (server_id, user_id)
        VALUES (:server_id, :user_id)",
        params! {
            "server_id" => server_id,
            "user_id" => user_id,
        },
    )
    .unwrap();

    Ok(warp::reply())
}

pub async fn search(
    json_data: ServerSearchData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let query = json_data.query;

    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec(
        r#"SELECT id, name, invite_code FROM server WHERE public = true AND name LIKE CONCAT("%", :query, "%")"#,
        params! {
            "query" => query,
        },
    )
    .unwrap();

    let mut server_name_list: Vec<SearchData> = Vec::new();
    for row in result {
        let (id, name, invite_code): (u64, String, String) = mysql::from_row(row);
        server_name_list.push(SearchData {
            id,
            name,
            invite_code,
        });
    }

    Ok(warp::reply::json(&server_name_list))
}

pub async fn get_invite_code(
    auth: AuthDetail,
    json_data: ServerInviteData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let user_id = auth.id;
    let server_id = json_data.id;

    // check if user has authority
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec(
        "SELECT * FROM user_server_relationship WHERE server_id = :server_id AND user_id = :user_id",
        params! {
            "server_id" => server_id,
            "user_id" => user_id,
        },
    )
    .unwrap();

    if result.is_empty() {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // get invite code from database
    let result: Vec<Row> = conn
        .exec(
            "SELECT invite_code FROM server WHERE id = :id",
            params! {
                "id" => server_id,
            },
        )
        .unwrap();

    let invite_code: String = mysql::from_row(result[0].clone());
    let response = InviteCodeData {
        invite_code: invite_code,
    };

    Ok(warp::reply::json(&response))
}

pub async fn create(
    auth: AuthDetail,
    json_data: ServerCreateData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let server_name = json_data.name;
    let server_public = json_data.public;
    let user_id = auth.id;

    // check if server_name lenght is appropriate
    if server_name.len() > 32 || server_name.len() == 0 {
        let invalid_params_vec: Vec<InvalidParamsDetail> = vec![InvalidParamsDetail {
            name: "name".to_string(),
            reason: "length out of range".to_string(),
        }];
        return Err(warp::reject::custom(ApiError::NotProcessable(
            invalid_params_vec,
        )));
    }

    // add server info to server table
    let mut conn = database.pool.get_conn().unwrap();
    conn.exec::<Row, _, _>(
        "INSERT INTO server (name, public, invite_code) VALUES (:name, :public, :invite_code)",
        params! {
            "name" => server_name,
            "public" => server_public,
            "invite_code" => "UNKNOWN".to_string(),
        },
    )
    .unwrap();

    let result: Vec<Row> = conn.exec("SELECT LAST_INSERT_ID()", ()).unwrap();
    let server_id: u64 = mysql::from_row(result[0].clone());

    // add authority info to user_server_relationship table
    conn.exec::<Row, _, _>(
        "INSERT INTO
        user_server_relationship (server_id, user_id)
        VALUES (:server_id, :user_id)",
        params! {
            "server_id" => server_id,
            "user_id" => user_id,
        },
    )
    .unwrap();

    // get invite code seed
    let result: Vec<Row> = conn.exec("SELECT seed FROM config", ()).unwrap();
    let seed: u64 = mysql::from_row(result[0].clone());

    // update invite code
    let generator = BlockId::new(Alphabet::alphanumeric(), seed as u128, 8);
    let invite_code = generator.encode_string(server_id).unwrap();
    conn.exec::<Row, _, _>(
        "UPDATE server SET invite_code = :invite_code WHERE id = :id",
        params! {
            "invite_code" => invite_code,
            "id" => server_id,
        },
    )
    .unwrap();

    Ok(warp::reply())
}

pub async fn delete(
    auth: AuthDetail,
    json_data: ServerDeleteData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let user_id = auth.id;
    let server_id = json_data.id;

    // check if user has authority
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec(
        "SELECT * FROM user_server_relationship WHERE server_id = :server_id AND user_id = :user_id",
        params! {
            "server_id" => server_id,
            "user_id" => user_id,
        },
    )
    .unwrap();

    if result.is_empty() {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // delete server form server table
    conn.exec::<Row, _, _>(
        "DELETE FROM server WHERE id = :server_id",
        params! {
            "server_id" => server_id,
        },
    )
    .unwrap();

    // delete server from user_server_relationship table
    conn.exec::<Row, _, _>(
        "DELETE FROM user_server_relationship WHERE server_id = :server_id",
        params! {
            "server_id" => server_id,
        },
    )
    .unwrap();

    Ok(warp::reply())
}

pub async fn modify(
    auth: AuthDetail,
    json_data: ServerModifyData,
    database: Database,
) -> Result<impl warp::Reply, Rejection> {
    let user_id = auth.id;
    let server_id = json_data.id;
    let server_name = json_data.name;
    let public = json_data.public;

    // check if user has authority
    let mut conn = database.pool.get_conn().unwrap();
    let result: Vec<Row> = conn.exec(
        "SELECT * FROM user_server_relationship WHERE server_id = :server_id AND user_id = :user_id",
        params! {
            "server_id" => server_id,
            "user_id" => user_id,
        },
    )
    .unwrap();

    if result.is_empty() {
        return Err(warp::reject::custom(ApiError::NotAuthorized));
    }

    // modify server info
    conn.exec::<Row, _, _>(
        "UPDATE server SET name = :name, public = :public WHERE id = :id",
        params! {
            "name" => server_name,
            "public" => public,
            "id" => server_id,
        },
    )
    .unwrap();

    Ok(warp::reply())
}

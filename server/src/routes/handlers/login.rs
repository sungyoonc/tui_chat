use std::convert::Infallible;
use warp::reply::Reply;
use crate::routes::*;
// use serde_json::json;
// use warp::http::Response;

pub async fn login(json_data: LoginData) -> Result<impl warp::Reply, Infallible> {
    println!("{:?}", json_data);
    if json_data == (LoginData {
        id: String::from("my_id"),
        pw: String::from("my_pw"),
    }) {
        Ok(warp::reply::json(&json_data).into_response())
    }
    else {
        Ok(warp::reply::json(&json_data).into_response())
    }
}

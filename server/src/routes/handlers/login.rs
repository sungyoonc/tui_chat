use std::convert::Infallible;
use crate::routes::*;
// use serde_json::json;
// use warp::http::Response;

pub async fn login(json_data: LoginData) -> Result<impl warp::Reply, Infallible> {
    println!("{:?}", json_data);
    if json_data == (LoginData {
        id: String::from("my_id"),
        pw: String::from("my_pw"),
    }) {
        Ok(warp::reply())
    }
    else {
        // Ok(Response::builder().body(json!(json_data).to_string()).unwrap())
        Ok(warp::reply())
    }
}

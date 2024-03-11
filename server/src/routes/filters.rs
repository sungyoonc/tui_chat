use crate::routes::handlers;
use warp::Filter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginData {
    pub id: String,
    pub pw: String,
}

pub fn apis() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    health_check().or(login())
}
pub fn health_check() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("health")
        .and(warp::get())
        .and_then(handlers::health_check)
}

pub fn login() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("login")
        .and(warp::post())
        .and(json_body())
        .and_then(handlers::login)
}

fn json_body() -> impl Filter<Extract = (LoginData,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
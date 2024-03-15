use crate::routes::handlers;
use warp::Filter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginData {
    pub username: String,
    pub pw: String,
    pub remember: bool,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct RefreshData {
    pub refresh_token: String,
}

pub fn apis() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    health_check().or(auth())
}
pub fn health_check() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("health")
        .and(warp::get())
        .and_then(handlers::health_check)
}

pub fn auth() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    let prefix = warp::path("auth");

    let login = warp::path("login")
        .and(warp::post())
        .and(login_data_json_body())
        .and_then(handlers::auth::login);

    let refresh = warp::path("refresh")
        .and(warp::post())
        .and(refresh_data_json_body())
        .and_then(handlers::auth::refresh);

    prefix.and(login.or(refresh))
}

fn login_data_json_body() -> impl Filter<Extract = (LoginData, ), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn refresh_data_json_body()-> impl Filter<Extract = (RefreshData, ), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
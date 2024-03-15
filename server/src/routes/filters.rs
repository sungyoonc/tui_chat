use crate::routes::handlers;
use crate::db::Db;
use warp::Filter;
use serde::{Deserialize, Serialize};
use mysql::Pool;

#[derive(Clone, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub pw: String,
    pub remember: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RefreshData {
    pub refresh_token: String,
}

#[derive(Clone)]
pub struct DB {
    pub pool: Pool
}

impl DB {
    pub fn new() -> DB {
        DB {pool: Db::new().pool}
    }

    pub fn apis(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        Self::health_check().or(Self::auth(db))
    }

    pub fn health_check() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
    {
        warp::path!("health")
            .and(warp::get())
            .and_then(handlers::health_check)
    }

    pub fn auth(db: DB) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
    {
        let prefix = warp::path("auth");

        let login = warp::path("login")
            .and(warp::post())
            .and(Self::login_data_json_body())
            .and(Self::with_db(db.clone()))
            .and_then(handlers::auth::login);

        let refresh = warp::path("refresh")
            .and(warp::post())
            .and(Self::refresh_data_json_body())
            .and(Self::with_db(db))
            .and_then(handlers::auth::refresh);

        prefix.and(login.or(refresh))
    }

    fn login_data_json_body() -> impl Filter<Extract = (LoginData, ), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn refresh_data_json_body()-> impl Filter<Extract = (RefreshData, ), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }
}
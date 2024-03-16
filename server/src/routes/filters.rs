use crate::routes::handlers;
use crate::db::Db;
use mysql::Pool;
use serde::{Deserialize, Serialize};
use warp::Filter;

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

pub struct Api {
    pool: Pool,
}

impl Api {
    pub fn new() -> Self {
        Self {
            pool: Db::new().pool,
        }
    }

    pub fn routes(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        self.health_check().or(self.auth())
    }

    pub fn health_check(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("health")
            .and(warp::get())
            .and_then(handlers::health_check)
    }

    pub fn auth(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let prefix = warp::path("auth");

        let login = warp::path("login")
            .and(warp::post())
            .and(login_data_json_body())
            .and(self.with_db())
            .and_then(handlers::auth::login);

        let refresh = warp::path("refresh")
            .and(warp::post())
            .and(refresh_data_json_body())
            .and(self.with_db())
            .and_then(handlers::auth::refresh);

        prefix.and(login.or(refresh))
    }

    fn with_db(&self) -> impl Filter<Extract = (Pool,), Error = std::convert::Infallible> + Clone {
        let db_pool = self.pool.clone();
        warp::any().map(move || db_pool.clone())
    }
}

fn login_data_json_body() -> impl Filter<Extract = (LoginData,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn refresh_data_json_body() -> impl Filter<Extract = (RefreshData,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

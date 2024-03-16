use crate::configuration::Settings;
use crate::db::Database;
use crate::routes::handlers;

use serde::{Deserialize, Serialize};
use warp::hyper::{Response, StatusCode};
use warp::reply::Reply;
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

#[derive(Serialize)]
struct RejectionDetails {
    title: String,
    status: u16,
}

#[derive(Debug)]
pub enum ApiError {
    NotAuthorized,
}

impl warp::reject::Reject for ApiError {}

pub struct Api {
    pub database: Database,
}

impl Api {
    pub fn new(settings: Settings) -> Self {
        let database = Database::new(&settings.database);
        database.db_setup();
        Self {
            database,
        }
    }

    pub fn routes(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = std::convert::Infallible> + Clone {
        self.health_check()
            .or(self.auth())
            .recover(Self::handle_rejection)
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

    pub async fn handle_rejection(
        err: warp::Rejection,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let status: StatusCode;
        let title: &str;

        if err.is_not_found() {
            title = "Not Found";
            status = StatusCode::NOT_FOUND;
        } else if err
            .find::<warp::filters::body::BodyDeserializeError>()
            .is_some()
        {
            // When the body could not be deserialized correctly
            title = "Bad Request";
            status = StatusCode::BAD_REQUEST;
        } else {
            title = "Unhandled Rejection";
            status = StatusCode::INTERNAL_SERVER_ERROR;
        }

        let json = RejectionDetails {
            title: title.to_string(),
            status: status.as_u16(),
        };

        let res = Response::builder()
            .status(status)
            .header(warp::http::header::CONTENT_TYPE, "application/problem+json")
            .body(warp::hyper::Body::from(
                serde_json::to_vec(&json).expect("Failed to serialize rejeciton details."),
            ))
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        Ok(res)
    }

    fn with_db(
        &self,
    ) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
        let database = self.database.clone();
        warp::any().map(move || database.clone())
    }
}

fn login_data_json_body() -> impl Filter<Extract = (LoginData,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn refresh_data_json_body() -> impl Filter<Extract = (RefreshData,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

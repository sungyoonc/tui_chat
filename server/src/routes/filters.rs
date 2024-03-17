use crate::configuration::Settings;
use crate::db::Database;
use crate::models::chat::Connections;
use crate::routes::handlers;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Clone, Deserialize)]
pub struct SignupData {
    pub username: String,
    pub pw: String,
}

#[derive(Clone, Deserialize)]
pub struct LogoutData {
    pub session: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct InvalidParamsDetail {
    pub name: String,
    pub reason: String,
}

impl InvalidParamsDetail {
    pub fn new(name: String, reason: String) -> Self {
        Self { name, reason }
    }
}

#[derive(Debug, Serialize)]
struct RejectionDetails {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    problem_type: Option<String>,
    title: String,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(rename = "invalid-params", skip_serializing_if = "Vec::is_empty")]
    invalid_params: Vec<InvalidParamsDetail>,
}

#[derive(Debug)]
pub enum ApiError {
    NotAuthorized,
    NotProcessable(Vec<InvalidParamsDetail>),
    InvalidQuery,
}

impl warp::reject::Reject for ApiError {}

pub struct Api {
    pub database: Database,
    pub ws_connections: Connections,
}

impl Api {
    pub fn new(settings: Settings, connections: Connections) -> Self {
        let database = Database::new(&settings.database);
        database.db_setup();
        Self {
            database,
            ws_connections: connections,
        }
    }

    pub async fn routes(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = std::convert::Infallible> + Clone {
        self.health_check()
            .or(self.auth())
            .or(self.chat().await)
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

        let signup = warp::path("signup")
            .and(warp::post())
            .and(signup_data_json_body())
            .and(self.with_db())
            .and_then(handlers::auth::signup);

        let logout = warp::path("logout")
            .and(warp::post())
            .and(logout_data_json_body())
            .and(self.with_db())
            .and_then(handlers::auth::logout);

        prefix.and(login.or(refresh).or(signup).or(logout))
    }

    pub async fn chat(
        &self,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let prefix = warp::path("chat");

        let ws = warp::path!("ws")
            .and(warp::query::<HashMap<String, String>>())
            .and(warp::ws())
            .and(self.with_ws_connections())
            .and(self.with_db())
            .and_then(
                |query: HashMap<String, String>,
                 ws: warp::ws::Ws,
                 connections: Connections,
                 database: Database| async move {
                    let token = match query.get("token") {
                        Some(token) => token,
                        None => {
                            return Err(warp::reject::custom(ApiError::NotAuthorized));
                        }
                    };
                    let id = match database.check_session(token.clone()).await {
                        Some(id) => id,
                        None => {
                            return Err(warp::reject::custom(ApiError::NotAuthorized));
                        }
                    };

                    let channel = match query.get("channel") {
                        Some(channel) => channel,
                        None => {
                            return Err(warp::reject::custom(ApiError::InvalidQuery));
                        }
                    };
                    let channel = match channel.parse::<u64>() {
                        Ok(channel) => channel,
                        Err(_) => {
                            return Err(warp::reject::custom(ApiError::InvalidQuery));
                        }
                    };

                    let res = ws.on_upgrade(move |socket| {
                        handlers::chat::ws(socket, connections, database, id, channel)
                    });
                    Ok(res)
                },
            );
        prefix.and(ws)
    }

    pub async fn ensure_authentication(
        &self,
    ) -> impl Filter<Extract = (u64,), Error = warp::Rejection> + Clone {
        self.with_db()
            .and(warp::header::optional::<String>("Authorization"))
            .and_then(
                |database: Database, auth_header: Option<String>| async move {
                    if let Some(token) = auth_header {
                        if let Some(id) = database.check_session(token).await {
                            return Ok(id);
                        }
                    }

                    Err(warp::reject::custom(ApiError::NotAuthorized))
                },
            )
    }

    pub async fn handle_rejection(
        err: warp::Rejection,
    ) -> Result<impl warp::Reply, std::convert::Infallible> {
        let status: StatusCode;
        let title: &str;
        let mut invalid_params: Vec<InvalidParamsDetail> = Vec::new();

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
        } else if let Some(e) = err.find::<ApiError>() {
            match e {
                ApiError::NotAuthorized => {
                    title = "Unauthorized";
                    status = StatusCode::UNAUTHORIZED;
                }
                ApiError::NotProcessable(invalid_params_vec) => {
                    title = "";
                    status = StatusCode::CONFLICT;
                    invalid_params.extend(invalid_params_vec.clone());
                }
                ApiError::InvalidQuery => {
                    title = "Bad Request";
                    status = StatusCode::BAD_REQUEST;
                }
            }
        } else {
            title = "Unhandled Rejection";
            status = StatusCode::INTERNAL_SERVER_ERROR;
        }

        let json = RejectionDetails {
            title: title.to_string(),
            status: status.as_u16(),
            problem_type: None,
            detail: None,
            instance: None,
            invalid_params,
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

    fn with_ws_connections(
        &self,
    ) -> impl Filter<Extract = (Connections,), Error = std::convert::Infallible> + Clone {
        let connections = self.ws_connections.clone();
        warp::any().map(move || connections.clone())
    }
}

fn login_data_json_body() -> impl Filter<Extract = (LoginData,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn refresh_data_json_body() -> impl Filter<Extract = (RefreshData,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn signup_data_json_body() -> impl Filter<Extract = (SignupData,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn logout_data_json_body() -> impl Filter<Extract = (LogoutData,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

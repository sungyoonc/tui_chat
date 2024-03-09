use crate::routes::handlers;
use warp::Filter;

pub fn apis() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    health_check()
}
pub fn health_check() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("health")
        .and(warp::get())
        .and_then(handlers::health_check)
}

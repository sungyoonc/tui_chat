use std::convert::Infallible;

pub async fn health_check() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply())
}

use std::future::Future;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Server;

use crate::routes;

fn get_server(
) -> Server<impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone> {
    warp::serve(routes::apis())
}

pub fn run(listener: TcpListener) -> impl Future<Output = ()> {
    let stream = TcpListenerStream::new(listener);
    get_server().serve_incoming(stream)
}

pub fn run_with_graceful_shutdown(
    listener: TcpListener,
    signal: impl Future<Output = ()> + Send + 'static,
) -> impl Future<Output = ()> {
    let stream = TcpListenerStream::new(listener);
    get_server().serve_incoming_with_graceful_shutdown(stream, signal)
}

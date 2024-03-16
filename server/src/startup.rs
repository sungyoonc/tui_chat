use std::future::Future;

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use warp::Server;

use crate::configuration::Settings;
use crate::routes::Api;

fn get_server(
    settings: Settings,
) -> Server<impl warp::Filter<Extract = (impl warp::Reply,), Error = std::convert::Infallible> + Clone> {
    let api = Api::new(settings);
    warp::serve(api.routes())
}

pub fn run(listener: TcpListener, settings: Settings) -> impl Future<Output = ()> {
    let stream = TcpListenerStream::new(listener);
    get_server(settings).serve_incoming(stream)
}

pub fn run_with_graceful_shutdown(
    listener: TcpListener,
    signal: impl Future<Output = ()> + Send + 'static,
    settings: Settings,
) -> impl Future<Output = ()> {
    let stream = TcpListenerStream::new(listener);
    get_server(settings).serve_incoming_with_graceful_shutdown(stream, signal)
}

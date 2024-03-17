use tui_chat_server::configuration::get_configuration;
use tui_chat_server::startup;

use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn spawn_server() -> (
    tokio::task::JoinHandle<()>,
    std::net::SocketAddr,
    CancellationToken,
) {
    let settings = get_configuration().expect("Failed to read configuration.");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port.");
    let address = listener.local_addr().unwrap();

    let cancel_token = CancellationToken::new();
    let cloned_cancel_token = cancel_token.clone();

    let server = startup::run_with_graceful_shutdown(
        listener,
        async move {
            cloned_cancel_token.cancelled().await;
        },
        settings.clone(),
    );

    (tokio::spawn(server.await), address, cancel_token)
}

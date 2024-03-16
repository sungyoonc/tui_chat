use tui_chat_server::configuration::get_configuration;
use tui_chat_server::startup;

use tokio::{net::TcpListener, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let settings = get_configuration().expect("Failed to read configuration.");

    let listener = TcpListener::bind((settings.bind.addr, settings.bind.port))
        .await
        .unwrap();

    let cancel_token = CancellationToken::new();
    let cloned_cancel_token = cancel_token.clone();

    let server = startup::run_with_graceful_shutdown(
        listener,
        async move {
            cloned_cancel_token.cancelled().await;
        },
        settings.clone(),
    );

    let server_task = tokio::spawn(server);

    // When SIGTERM or Ctrl-C is received, shutdown the server
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = sigterm.recv() => {},
        _ = signal::ctrl_c() => {},
    }
    eprintln!("Shutting Down.");
    cancel_token.cancel();
    server_task.await.unwrap();
}

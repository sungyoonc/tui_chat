use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tui_chat_server::startup;

#[tokio::test]
async fn test_health_check() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{}/health", address.port()))
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));

    // Shutdown the server
    cancel_token.cancel();
    server_task.await.unwrap();
}

async fn spawn_server() -> (
    tokio::task::JoinHandle<()>,
    std::net::SocketAddr,
    CancellationToken,
) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port.");
    let address = listener.local_addr().unwrap();

    let cancel_token = CancellationToken::new();
    let cloned_cancel_token = cancel_token.clone();

    let server = startup::run_with_graceful_shutdown(listener, async move {
        cloned_cancel_token.cancelled().await;
    });

    (tokio::spawn(server), address, cancel_token)
}

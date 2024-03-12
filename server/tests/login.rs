use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tui_chat_server::startup;
use std::collections::HashMap;

#[tokio::test]
async fn test_login() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("id", "my_id");
    map.insert("pw", "my_pw");

    let response = client
        .post(format!("http://127.0.0.1:{}/login", address.port()))
        .json(&map)
        .send()
        .await
        .expect("Failed to send request.");

    assert!(response.status().is_success());
    println!("{:?}", response.text().await);

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

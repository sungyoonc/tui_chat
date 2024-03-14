use test_util::spawn_server;

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

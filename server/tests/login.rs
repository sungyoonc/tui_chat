use test_util::spawn_server;
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
    assert_eq!(response.content_length(), Some(0));

    // Shutdown the server
    cancel_token.cancel();
    server_task.await.unwrap();
}


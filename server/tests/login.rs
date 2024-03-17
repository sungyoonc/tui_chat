use serde::Serialize;
use test_util::spawn_server;

#[derive(Serialize)]
pub struct LoginData {
    pub username: String,
    pub pw: String,
    pub remember: bool,
}

#[derive(Serialize)]
pub struct SignupData {
    pub username: String,
    pub pw: String,
}

#[derive(Serialize)]
pub struct LogoutData {
    pub session: String,
}

#[tokio::test]
async fn test_login() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = LoginData {
        username: "create_my_id".to_string(),
        pw: "creat_my_pw".to_string(),
        remember: true,
    };

    let response = client
        .post(format!("http://127.0.0.1:{}/auth/login", address.port()))
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

#[tokio::test]
async fn test_signup() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = SignupData {
        username: "create_my_id".to_string(),
        pw: "creat_my_pw".to_string(),
    };

    let response = client
        .post(format!("http://127.0.0.1:{}/auth/signup", address.port()))
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

#[tokio::test]
async fn test_logout() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = LogoutData {
        session: "92f9fef2906db2524cb5eb096619396f9b45c2f835e05ac174c7d90749ac9b1a".to_string(),
    };

    let response = client
        .post(format!("http://127.0.0.1:{}/auth/logout", address.port()))
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

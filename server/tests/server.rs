use serde::Serialize;
use test_util::spawn_server;

const SESSION: &str = "4b7275343789f043a75274f8301d325537e303cb4ac1bff6476a255815f60ac6";

#[derive(Clone, Serialize)]
pub struct ServerCreateData {
    pub name: String,
    pub public: bool,
}

#[derive(Clone, Serialize)]
pub struct ServerSearchData {
    pub query: String,
}

#[derive(Clone, Serialize)]
pub struct GetInviteCodeData {
    pub id: u64,
}

#[derive(Clone, Serialize)]
pub struct ServerJoinData {
    pub invite_code: String,
}

#[derive(Clone, Serialize)]
pub struct ServerModifyData {
    pub id: u64,
    pub name: String,
    pub public: bool,
}

#[derive(Clone, Serialize)]
pub struct ServerDeleteData {
    pub id: u64,
}

#[tokio::test]
async fn create_server() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = ServerCreateData {
        name: "test".to_string(),
        public: true,
    };

    let response = client
        .post(format!(
            "http://127.0.0.1:{}/server/manage/create",
            address.port()
        ))
        .header("Authorization", SESSION.to_string())
        .json(&map)
        .send()
        .await
        .expect("Failed to send request.");

    println!("{:?}", response);
    assert!(response.status().is_success());
    println!("{:?}", response.text().await);

    // Shutdown the server
    cancel_token.cancel();
    server_task.await.unwrap();
}

#[tokio::test]
async fn search_server() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = ServerSearchData {
        query: "test".to_string(),
    };

    let response = client
        .post(format!("http://127.0.0.1:{}/server/search", address.port()))
        .header("Authorization", SESSION.to_string())
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
async fn get_invite_code() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = GetInviteCodeData { id: 1 };

    let response = client
        .post(format!(
            "http://127.0.0.1:{}/server/get_invite_code",
            address.port()
        ))
        .header("Authorization", SESSION.to_string())
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
async fn join_server() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = ServerJoinData {
        invite_code: "8mWYjkcQ".to_string(),
    };

    let response = client
        .post(format!("http://127.0.0.1:{}/server/join", address.port()))
        .header("Authorization", SESSION.to_string())
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
async fn modify_server() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = ServerModifyData {
        id: 1,
        name: "test2".to_string(),
        public: false,
    };

    let response = client
        .post(format!(
            "http://127.0.0.1:{}/server/manage/modify",
            address.port()
        ))
        .header("Authorization", SESSION.to_string())
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
async fn delete_server() {
    let (server_task, address, cancel_token) = spawn_server().await;
    let client = reqwest::Client::new();

    let map = ServerDeleteData { id: 1 };

    let response = client
        .post(format!(
            "http://127.0.0.1:{}/server/manage/delete",
            address.port()
        ))
        .header("Authorization", SESSION.to_string())
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

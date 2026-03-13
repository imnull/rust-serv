use hyper::StatusCode;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use rust_serv::{Config, Server};

/// Test helper to start a server on a random port
async fn start_test_server(root: &str) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let config = Config {
        port,
        root: root.into(),
        ..Default::default()
    };

    let server = Server::new(config);
    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    (port, handle)
}

async fn make_request(path: &str, port: u16) -> reqwest::Response {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}{}", port, path);
    client.get(&url).send().await.unwrap()
}

#[tokio::test]
async fn test_server_starts() {
    let temp_dir = TempDir::new().unwrap();
    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;
    assert!(port > 0);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}", port);
    let result = tokio::time::timeout(Duration::from_secs(1), client.head(&url).send()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_root_with_index_html() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("index.html"), "<h1>Index Page</h1>").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert!(body.contains("Index Page"));
}

#[tokio::test]
async fn test_root_without_index_html() {
    let temp_dir = TempDir::new().unwrap();

    // With directory indexing disabled, should return 404
    let (port, _handle) = start_test_server_with_indexing(temp_dir.path().to_str().unwrap(), false).await;

    let response = make_request("/", port).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn start_test_server_with_indexing(root: &str, enable_indexing: bool) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let config = Config {
        port,
        root: root.into(),
        enable_indexing,
        ..Default::default()
    };

    let server = Server::new(config);
    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    (port, handle)
}

#[tokio::test]
async fn test_nonexistent_path_returns_404() {
    let temp_dir = TempDir::new().unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/nonexistent", port).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

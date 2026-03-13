use hyper::StatusCode;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use rust_serv::{Config, Server};

async fn start_test_server(root: &str, enable_indexing: bool) -> (u16, tokio::task::JoinHandle<()>) {
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

async fn make_request(path: &str, port: u16) -> reqwest::Response {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}{}", port, path);
    client.get(&url).send().await.unwrap()
}

#[tokio::test]
async fn test_directory_returns_html_list() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap(), true).await;

    let response = make_request("/", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert!(body.contains("<html>") || body.contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_directory_list_includes_files() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap(), true).await;

    let response = make_request("/", port).await;
    let body = response.text().await.unwrap();

    assert!(body.contains("test.txt"));
}

#[tokio::test]
async fn test_directory_list_hides_hidden_files() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "visible").unwrap();
    fs::write(temp_dir.path().join(".hidden.txt"), "hidden").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap(), true).await;

    let response = make_request("/", port).await;
    let body = response.text().await.unwrap();

    assert!(body.contains("visible.txt"));
    assert!(!body.contains(".hidden.txt"));
}

#[tokio::test]
async fn test_directory_list_with_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file.txt"), "file").unwrap();
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap(), true).await;

    let response = make_request("/", port).await;
    let body = response.text().await.unwrap();

    assert!(body.contains("file.txt"));
    assert!(body.contains("subdir"));
}

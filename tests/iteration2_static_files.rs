use hyper::StatusCode;
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::time::timeout;
use rust_serv::{Config, Server};

/// Test helper to start a server on a random port
async fn start_test_server(root: &str) -> (u16, tokio::task::JoinHandle<()>) {
    // Find a free port
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

    // Wait for server to start
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
async fn test_return_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/test.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "Hello, World!");
}

#[tokio::test]
async fn test_file_not_found_404() {
    let temp_dir = TempDir::new().unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/nonexistent.txt", port).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_correct_content_type_html() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.html");
    fs::write(&file_path, "<html></html>").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/test.html", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(content_type.starts_with("text/html"));
}

#[tokio::test]
async fn test_correct_content_type_css() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("style.css");
    fs::write(&file_path, "body {}").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/style.css", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(content_type.starts_with("text/css"));
}

#[tokio::test]
async fn test_correct_content_type_png() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("image.png");
    fs::write(&file_path, "fake png data").unwrap();

    let (port, _handle) = start_test_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/image.png", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap();
    assert!(content_type.starts_with("image/png"));
}

use hyper::StatusCode;
use std::fs;
use tempfile::TempDir;
use rust_serv::{Config, Server};

async fn start_server(root: &str) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let config = Config { port, root: root.into(), ..Default::default() };
    let server = Server::new(config);
    let handle = tokio::spawn(async move { let _ = server.run().await; });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

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
async fn test_etag_generated_for_files() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"Test ETag content".to_vec();
    fs::write(temp_dir.path().join("etag_test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/etag_test.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Check for ETag header
    let etag = response.headers().get("ETag")
        .and_then(|v| v.to_str().ok());
    assert!(etag.is_some(), "ETag header should be present");
}

#[tokio::test]
async fn test_accept_ranges_header() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"Test range content".to_vec();
    fs::write(temp_dir.path().join("range_test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/range_test.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Check for Accept-Ranges header
    let accept_ranges = response.headers().get("Accept-Ranges")
        .and_then(|v| v.to_str().ok());
    assert_eq!(accept_ranges, Some("bytes"));
}

#[tokio::test]
async fn test_content_length_header() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"Test content length".to_vec();
    fs::write(temp_dir.path().join("length_test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/length_test.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Check for Content-Length header
    let content_length = response.headers().get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());
    assert_eq!(content_length, Some(content.len()));
}

#[tokio::test]
async fn test_multiple_files_different_etags() {
    let temp_dir = TempDir::new().unwrap();

    // Create two different files with different content and sizes
    fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10)); // Small delay for different timestamps
    fs::write(temp_dir.path().join("file2.txt"), "content 2 different size").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response1 = make_request("/file1.txt", port).await;
    let response2 = make_request("/file2.txt", port).await;

    let etag1 = response1.headers().get("ETag").and_then(|v| v.to_str().ok());
    let etag2 = response2.headers().get("ETag").and_then(|v| v.to_str().ok());

    assert!(etag1.is_some());
    assert!(etag2.is_some());
    assert_ne!(etag1, etag2, "Different files should have different ETags");
}

#[tokio::test]
async fn test_directory_html_generation() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // Enable indexing in config
    drop(_handle);
    let config = Config {
        port,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        ..Default::default()
    };

    let server = Server::new(config);
    let handle = tokio::spawn(async move { let _ = server.run().await; });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let response = make_request("/", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert!(body.contains("Index of"));
    assert!(body.contains("file1.txt"));
    assert!(body.contains("file2.txt"));

    handle.abort();
}

#[tokio::test]
async fn test_config_options() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("config_test.txt"), "test").unwrap();

    // Test default config
    let config = Config::default();
    assert_eq!(config.port, 8080);
    assert_eq!(config.enable_indexing, true); // Default is true
    assert_eq!(config.enable_compression, true); // Default is true
    assert_eq!(config.log_level, "info");

    // Test custom config
    let custom_config = Config {
        port: 3000,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        enable_compression: true,
        log_level: "debug".into(),
        enable_tls: false,
        tls_cert: None,
        tls_key: None,
        connection_timeout_secs: 30,
        max_connections: 1000,
        enable_health_check: true,
        ..Default::default()
    };

    assert_eq!(custom_config.port, 3000);
    assert!(custom_config.enable_indexing);
    assert!(custom_config.enable_compression);
    assert_eq!(custom_config.log_level, "debug");
}

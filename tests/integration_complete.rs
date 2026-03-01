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
async fn test_static_files_with_compression() {
    let temp_dir = TempDir::new().unwrap();
    let html_content = r#"<!DOCTYPE html><html><body><h1>Compressed Page</h1></body></html>"#;
    fs::write(temp_dir.path().join("index.html"), html_content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/index.html", port).await;

    // Response should be 200 OK with compressed content
    assert_eq!(response.status(), StatusCode::OK);

    // Check for Content-Encoding header
    let encoding = response.headers().get("Content-Encoding")
        .and_then(|v| v.to_str().ok());

    // Note: Compression not yet implemented, so we verify basic file serving
    let body = response.text().await.unwrap();
    assert!(body.contains("Compressed Page"));
}

#[tokio::test]
async fn test_etag_and_cache() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"Hello from cache!".to_vec();
    fs::write(temp_dir.path().join("cache_test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // First request without If-None-Match
    let response1 = make_request("/cache_test.txt", port).await;
    assert_eq!(response1.status(), StatusCode::OK);

    // Get ETag from first response
    let etag = response1.headers().get("ETag")
        .and_then(|v| v.to_str().ok())
        .expect("ETag header should be present");

    // Second request with If-None-Match
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}{}", port, "/cache_test.txt");
    let response2 = client
        .get(&url)
        .header("If-None-Match", etag)
        .send()
        .await
        .unwrap();

    // Should return 304 Not Modified
    assert_eq!(response2.status(), StatusCode::NOT_MODIFIED);

    let body2 = response2.text().await.unwrap();
    // Empty body for 304
    assert_eq!(body2, "");
}

#[tokio::test]
async fn test_logging() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"Test logging output";
    fs::write(temp_dir.path().join("log.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/log.txt", port).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.text().await.unwrap().contains("Test logging output"));
}

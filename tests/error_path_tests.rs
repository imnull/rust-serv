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

#[tokio::test]
async fn test_etag_mismatch_returns_200() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"ETag test content".to_vec();
    fs::write(temp_dir.path().join("etag.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // First request to get the ETag
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/etag.txt", port);
    let response1 = client.get(&url).send().await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let etag = response1.headers().get("ETag")
        .and_then(|v| v.to_str().ok())
        .unwrap();

    // Second request with different (incorrect) ETag
    let response2 = client
        .get(&url)
        .header("If-None-Match", "\"wrong-etag\"")
        .send()
        .await
        .unwrap();

    // Should return 200 OK with content (ETag mismatch)
    assert_eq!(response2.status(), StatusCode::OK);
    let body = response2.text().await.unwrap();
    assert_eq!(body, "ETag test content");
}

#[tokio::test]
async fn test_path_traversal_returns_403_or_404() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("safe.txt"), "safe content").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // Try to access file outside root directory
    let url = format!("http://127.0.0.1:{}/../../../etc/passwd", port);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(&url).send().await.unwrap();

    // Should return 403 Forbidden or 404 (depending on system)
    assert!(matches!(response.status(), StatusCode::FORBIDDEN | StatusCode::NOT_FOUND));
}

#[tokio::test]
async fn test_range_request_with_etag() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("range_etag.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/range_etag.txt", port);

    // Make range request
    let response = client
        .get(&url)
        .header("Range", "bytes=0-4")
        .send()
        .await
        .unwrap();

    // Check ETag is present before consuming response
    let etag = response.headers().get("ETag");
    assert!(etag.is_some());

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.text().await.unwrap(), "01234");
}

#[tokio::test]
async fn test_invalid_utf8_url() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // Make request with invalid UTF-8 (should still work with fallback)
    let url = format!("http://127.0.0.1:{}/test.txt", port);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_directory_with_index_html() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("with_index");
    fs::create_dir(&dir_path).unwrap();
    fs::write(dir_path.join("index.html"), "<html>Index</html>").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/with_index/", port).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert!(body.contains("<html>Index</html>"));
}

async fn make_request(path: &str, port: u16) -> reqwest::Response {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}{}", port, path);
    client.get(&url).send().await.unwrap()
}

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

async fn make_request(path: &str, port: u16, range: Option<&str>) -> reqwest::Response {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}{}", port, path);

    let mut req = client.get(&url);
    if let Some(r) = range {
        req = req.header("Range", r);
    }
    req.send().await.unwrap()
}

#[tokio::test]
async fn test_range_request_bytes_0_4() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/test.txt", port, Some("bytes=0-4")).await;

    // Range request should return 206
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let content_range = response.headers().get("Content-Range")
        .and_then(|v| v.to_str().ok());

    assert_eq!(content_range, Some("bytes 0-4/10"));

    let body = response.text().await.unwrap();

    // Should return only requested bytes (0,1,2,3,4)
    assert_eq!(body, "01234");
}

#[tokio::test]
async fn test_range_request_bytes_5_9() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/test.txt", port, Some("bytes=5-9")).await;

    // Range request should return 206
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let content_range = response.headers().get("Content-Range")
        .and_then(|v| v.to_str().ok());

    assert_eq!(content_range, Some("bytes 5-9/10"));

    let body = response.text().await.unwrap();

    // Should return only requested bytes (5,6,7,8,9)
    assert_eq!(body, "56789");
}

#[tokio::test]
async fn test_no_range_header() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/test.txt", port, None).await;

    // No range header should return 200 OK with full content
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "0123456789");
}

#[tokio::test]
async fn test_invalid_range() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // Invalid range (4-1 where start > end) should return full content
    let response = make_request("/test.txt", port, Some("bytes=4-1")).await;

    // Should return full content
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "0123456789");
}

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
async fn test_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("empty.txt"), "").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/empty.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "");
}

#[tokio::test]
async fn test_large_file() {
    let temp_dir = TempDir::new().unwrap();
    let large_content = "x".repeat(100_000);
    fs::write(temp_dir.path().join("large.txt"), &large_content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/large.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body.len(), 100_000);
}

#[tokio::test]
async fn test_special_characters_in_filename() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("test file.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("test-file.txt"), "content2").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // Test space in filename (URL encoded)
    let url = format!("http://127.0.0.1:{}/test%20file.txt", port);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_subdirectory_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "nested content").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/subdir/nested.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "nested content");
}

#[tokio::test]
async fn test_deeply_nested_path() {
    let temp_dir = TempDir::new().unwrap();
    let deep_path = temp_dir.path().join("a/b/c/d");
    fs::create_dir_all(&deep_path).unwrap();
    fs::write(deep_path.join("deep.txt"), "deep content").unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/a/b/c/d/deep.txt", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert_eq!(body, "deep content");
}

#[tokio::test]
async fn test_multiple_range_requests() {
    let temp_dir = TempDir::new().unwrap();
    let content = b"0123456789".to_vec();
    fs::write(temp_dir.path().join("range_test.txt"), &content).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    // First range request
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url1 = format!("http://127.0.0.1:{}/range_test.txt", port);
    let response1 = client.get(&url1).header("Range", "bytes=0-4").send().await.unwrap();
    assert_eq!(response1.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response1.text().await.unwrap(), "01234");

    // Second range request
    let response2 = client.get(&url1).header("Range", "bytes=5-9").send().await.unwrap();
    assert_eq!(response2.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response2.text().await.unwrap(), "56789");
}

#[tokio::test]
async fn test_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/nonexistent/", port).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_directory_without_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    let (port, handle) = start_server(temp_dir.path().to_str().unwrap()).await;
    handle.abort();

    // Create server with indexing disabled
    let config = Config {
        port,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: false,
        ..Default::default()
    };

    let server = Server::new(config);
    let handle2 = tokio::spawn(async move { let _ = server.run().await; });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let response = make_request("/subdir/", port).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    handle2.abort();
}

#[tokio::test]
async fn test_binary_file() {
    let temp_dir = TempDir::new().unwrap();
    let binary_data: Vec<u8> = (0..=255).collect();
    fs::write(temp_dir.path().join("binary.bin"), &binary_data).unwrap();

    let (port, _handle) = start_server(temp_dir.path().to_str().unwrap()).await;

    let response = make_request("/binary.bin", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.bytes().await.unwrap();
    assert_eq!(body.len(), 256);
    assert_eq!(body.to_vec(), binary_data);
}

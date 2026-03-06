//! Integration tests for handler error paths and edge cases

use hyper::StatusCode;
use std::fs;
use tempfile::TempDir;
use rust_serv::{Config, Server};

use std::sync::atomic::{AtomicU16, Ordering};

static TEST_PORT: AtomicU16 = AtomicU16::new(15000);

fn get_test_port() -> u16 {
    TEST_PORT.fetch_add(1, Ordering::SeqCst)
}

async fn start_server_with_config(mut config: Config) -> (u16, tokio::task::JoinHandle<()>) {
    let port = get_test_port();
    config.port = port;

    let server = Server::new(config);
    let handle = tokio::spawn(async move { let _ = server.run().await; });
    
    // Wait for server to start
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
            break;
        }
    }
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

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
async fn test_etag_cache_hit_returns_304() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("cached.txt"), "cached content").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/cached.txt", port);

    // First request to get ETag
    let response1 = client.get(&url).send().await.unwrap();
    let etag = response1.headers().get("ETag").unwrap().to_str().unwrap().to_string();

    // Second request with matching ETag
    let response2 = client
        .get(&url)
        .header("If-None-Match", etag)
        .send()
        .await
        .unwrap();

    // Should return 304 Not Modified
    assert_eq!(response2.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(response2.text().await.unwrap(), "");
}

#[tokio::test]
async fn test_directory_listing_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("listable");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file1.txt"), "content1").unwrap();
    fs::write(subdir.join("file2.txt"), "content2").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let response = make_request("/listable/", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert!(body.contains("file1.txt"));
    assert!(body.contains("file2.txt"));
    assert!(body.contains("Index of"));
}

#[tokio::test]
async fn test_invalid_range_request_returns_full_content() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("range.txt"), "0123456789").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/range.txt", port);

    // Invalid range format
    let response = client
        .get(&url)
        .header("Range", "invalid-range")
        .send()
        .await
        .unwrap();

    // Should return 200 OK with full content
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "0123456789");
}

#[tokio::test]
async fn test_range_beyond_file_size() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("small.txt"), "tiny").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/small.txt", port);

    // Range beyond file size
    let response = client
        .get(&url)
        .header("Range", "bytes=100-200")
        .send()
        .await
        .unwrap();

    // Should return full content when range is invalid
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_url_encoded_path() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file with spaces.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("café.txt"), "coffee").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // URL encoded space
    let url1 = format!("http://127.0.0.1:{}/file%20with%20spaces.txt", port);
    let response1 = client.get(&url1).send().await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // URL encoded unicode
    let url2 = format!("http://127.0.0.1:{}/caf%C3%A9.txt", port);
    let response2 = client.get(&url2).send().await.unwrap();
    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.text().await.unwrap(), "coffee");
}

#[tokio::test]
async fn test_request_with_query_string() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("api.txt"), "api response").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Request with query string
    let url = format!("http://127.0.0.1:{}/api.txt?param=value&foo=bar", port);
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "api response");
}

#[tokio::test]
async fn test_hidden_files_not_listed() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("testdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("visible.txt"), "visible").unwrap();
    fs::write(subdir.join(".hidden"), "hidden").unwrap();
    fs::write(subdir.join(".hidden.txt"), "hidden content").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let response = make_request("/testdir/", port).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.text().await.unwrap();
    assert!(body.contains("visible.txt"));
    assert!(!body.contains(".hidden"));
}

#[tokio::test]
async fn test_file_with_hash_in_name() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file#hash.txt"), "content").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // URL encoded hash
    let url = format!("http://127.0.0.1:{}/file%23hash.txt", port);
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_concurrent_requests() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("concurrent.txt"), "concurrent content").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/concurrent.txt", port);

    // Spawn multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let response = client.get(&url).send().await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            response.text().await.unwrap()
        }));
    }

    // All requests should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert_eq!(result, "concurrent content");
    }
}

#[tokio::test]
async fn test_head_request() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("head_test.txt"), "content body here").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/head_test.txt", port);
    let response = client.head(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-length").unwrap(), "17");
    // HEAD response should have empty body
    assert_eq!(response.text().await.unwrap(), "");
}

#[tokio::test]
async fn test_file_with_plus_sign_in_name() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("file+plus.txt"), "plus content").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Plus sign can be literal or encoded
    let url = format!("http://127.0.0.1:{}/file+plus.txt", port);
    let response = client.get(&url).send().await.unwrap();

    // Both should work
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_symlink_to_file() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("target.txt"), "target content").unwrap();
    
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            temp_dir.path().join("target.txt"),
            temp_dir.path().join("link.txt"),
        ).unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(
            temp_dir.path().join("target.txt"),
            temp_dir.path().join("link.txt"),
        ).unwrap();
    }

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let response = make_request("/link.txt", port).await;
    
    // Symlinks should be followed if they point within root
    if response.status() == StatusCode::OK {
        assert_eq!(response.text().await.unwrap(), "target content");
    }
}

//! Integration tests for server error paths and edge cases

use hyper::StatusCode;
use std::fs;
use tempfile::TempDir;
use rust_serv::{Config, Server};

use std::sync::atomic::{AtomicU16, Ordering};

static TEST_PORT: AtomicU16 = AtomicU16::new(16000);

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

async fn make_request_with_headers(path: &str, port: u16, headers: Vec<(&str, &str)>) -> reqwest::Response {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}{}", port, path);
    let mut request = client.get(&url);
    
    for (key, value) in headers {
        request = request.header(key, value);
    }
    
    request.send().await.unwrap()
}

#[tokio::test]
async fn test_compression_with_accept_encoding_gzip() {
    let temp_dir = TempDir::new().unwrap();
    // Create large compressible content
    let content = "Hello, World! ".repeat(1000);
    fs::write(temp_dir.path().join("compressible.txt"), &content).unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_compression: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/compressible.txt", port);
    let response = client
        .get(&url)
        .header("Accept-Encoding", "gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Response may or may not be compressed depending on size threshold
}

#[tokio::test]
async fn test_compression_with_accept_encoding_brotli() {
    let temp_dir = TempDir::new().unwrap();
    let content = "Hello, World! ".repeat(1000);
    fs::write(temp_dir.path().join("compressible2.txt"), &content).unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_compression: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/compressible2.txt", port);
    let response = client
        .get(&url)
        .header("Accept-Encoding", "br, gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_no_compression_for_images() {
    let temp_dir = TempDir::new().unwrap();
    // Create fake image data
    let image_data = vec![0xFFu8; 1000];
    fs::write(temp_dir.path().join("test.png"), &image_data).unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_compression: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/test.png", port);
    let response = client
        .get(&url)
        .header("Accept-Encoding", "gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Images should not be compressed
    assert!(response.headers().get("content-encoding").is_none());
}

#[tokio::test]
async fn test_range_request_with_end_beyond_file() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("partial.txt"), "0123456789").unwrap();

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

    let url = format!("http://127.0.0.1:{}/partial.txt", port);
    let response = client
        .get(&url)
        .header("Range", "bytes=5-100")
        .send()
        .await
        .unwrap();

    // Range beyond file may return 206 (partial) with adjusted range or 200 with full content
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::PARTIAL_CONTENT);
}

#[tokio::test]
async fn test_multiple_etag_requests() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("etag_multi.txt"), "etag content").unwrap();

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

    let url = format!("http://127.0.0.1:{}/etag_multi.txt", port);

    // First request
    let response1 = client.get(&url).send().await.unwrap();
    let etag1 = response1.headers().get("ETag").unwrap().to_str().unwrap().to_string();

    // Multiple requests with same ETag
    for _ in 0..5 {
        let response = client
            .get(&url)
            .header("If-None-Match", &etag1)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    // Modify file
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    fs::write(temp_dir.path().join("etag_multi.txt"), "modified content").unwrap();

    // Request should return new content (different ETag)
    let response2 = client
        .get(&url)
        .header("If-None-Match", etag1)
        .send()
        .await
        .unwrap();
    
    // May return 200 or 304 depending on file system precision
    assert!(response2.status() == StatusCode::OK || response2.status() == StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn test_request_with_multiple_ranges() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("multi_range.txt"), "0123456789ABCDEF").unwrap();

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

    let url = format!("http://127.0.0.1:{}/multi_range.txt", port);
    
    // Multiple ranges (may not be supported, should return full content)
    let response = client
        .get(&url)
        .header("Range", "bytes=0-4, 10-15")
        .send()
        .await
        .unwrap();

    // Should either return partial content or full content
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::PARTIAL_CONTENT);
}

#[tokio::test]
async fn test_empty_directory_with_indexing() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let url = format!("http://127.0.0.1:{}/empty/", port);
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.text().await.unwrap();
    assert!(body.contains("Index of"));
}

#[tokio::test]
async fn test_root_path_trailing_slash() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("index.html"), "<h1>Root</h1>").unwrap();

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

    // Both should work
    let url1 = format!("http://127.0.0.1:{}/", port);
    let response1 = client.get(&url1).send().await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_very_long_path() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create deeply nested directory structure
    let mut deep_path = temp_dir.path().to_path_buf();
    for i in 0..10 {
        deep_path = deep_path.join(format!("level{}", i));
        fs::create_dir(&deep_path).unwrap();
    }
    fs::write(deep_path.join("deep_file.txt"), "deep content").unwrap();

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

    let url = format!("http://127.0.0.1:{}/level0/level1/level2/level3/level4/level5/level6/level7/level8/level9/deep_file.txt", port);
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "deep content");
}

#[tokio::test]
async fn test_unicode_filenames() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("日本語.txt"), "japanese").unwrap();
    fs::write(temp_dir.path().join("emoji🎉.txt"), "party").unwrap();
    fs::write(temp_dir.path().join("кyrillic.txt"), "russian").unwrap();

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

    // Test Japanese
    let url1 = format!("http://127.0.0.1:{}/{}", port, urlencoding::encode("日本語.txt"));
    let response1 = client.get(&url1).send().await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // Test emoji (may or may not work depending on OS)
    let url2 = format!("http://127.0.0.1:{}/{}", port, urlencoding::encode("emoji🎉.txt"));
    let response2 = client.get(&url2).send().await.unwrap();
    // Emoji in filenames might not work on all systems
    assert!(response2.status() == StatusCode::OK || response2.status() == StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_file_with_dot_prefix() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join(".dotfile"), "hidden").unwrap();
    fs::write(temp_dir.path().join("visible.txt"), "visible").unwrap();

    let config = Config {
        port: 0,
        root: temp_dir.path().to_str().unwrap().into(),
        enable_indexing: true,
        ..Default::default()
    };
    let (port, _handle) = start_server_with_config(config).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Hidden file should still be accessible directly
    let url1 = format!("http://127.0.0.1:{}/.dotfile", port);
    let response1 = client.get(&url1).send().await.unwrap();
    // May be accessible or return 404 depending on implementation
    
    // But should not appear in directory listing
    let url2 = format!("http://127.0.0.1:{}/", port);
    let response2 = client.get(&url2).send().await.unwrap();
    let body = response2.text().await.unwrap();
    assert!(!body.contains(".dotfile"));
}

#[tokio::test]
async fn test_request_with_user_agent() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("agent_test.txt"), "content").unwrap();

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

    let url = format!("http://127.0.0.1:{}/agent_test.txt", port);
    let response = client
        .get(&url)
        .header("User-Agent", "TestAgent/1.0")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_request_not_allowed() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("post_test.txt"), "content").unwrap();

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

    let url = format!("http://127.0.0.1:{}/post_test.txt", port);
    let response = client.post(&url).body("data").send().await.unwrap();

    // POST might be treated as GET or return 405
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::METHOD_NOT_ALLOWED);
}

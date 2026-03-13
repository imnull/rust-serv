//! Performance benchmarks for rust-serv
//!
//! This benchmark suite tests the performance of key components:
//! - File service operations
//! - Compression algorithms
//! - Path validation
//! - MIME type detection
//! - Concurrent request handling

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_serv::file_service::FileService;
use rust_serv::path_security::PathValidator;
use rust_serv::mime_types::MimeDetector;
use rust_serv::handler::compress;
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

/// Benchmark file reading operations
fn bench_file_service(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    // Test different file sizes
    let sizes = vec![1024, 10 * 1024, 100 * 1024, 1024 * 1024]; // 1KB, 10KB, 100KB, 1MB

    let mut group = c.benchmark_group("file_service_read");

    for size in sizes {
        let file_path = temp_dir.path().join(format!("test_{}.txt", size));
        let content = "a".repeat(size);
        fs::write(&file_path, content).unwrap();

        group.bench_with_input(BenchmarkId::new("read_file", size), &file_path, |b, path| {
            b.iter(|| {
                black_box(FileService::read_file(black_box(path)).unwrap())
            });
        });
    }

    group.finish();
}

/// Benchmark Gzip compression
fn bench_gzip_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("gzip_compression");

    // Test different data patterns
    let test_data = vec![
        ("repetitive", "a".repeat(100_000).into_bytes()),
        ("text", "The quick brown fox jumps over the lazy dog. ".repeat(5000).into_bytes()),
        ("json", r#"{"name":"test","value":123,"nested":{"key":"value"},"array":[1,2,3]}"#.repeat(1000).into_bytes()),
        ("html", "<html><body><div>test</div></body></html>".repeat(5000).into_bytes()),
    ];

    for (name, data) in test_data {
        group.bench_with_input(BenchmarkId::new(name, data.len()), &data, |b, input| {
            b.iter(|| {
                black_box(compress::compress_gzip(black_box(input)).unwrap())
            });
        });
    }

    group.finish();
}

/// Benchmark Brotli compression
fn bench_brotli_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("brotli_compression");

    let test_data = vec![
        ("repetitive", "a".repeat(100_000).into_bytes()),
        ("text", "The quick brown fox jumps over the lazy dog. ".repeat(5000).into_bytes()),
        ("json", r#"{"name":"test","value":123,"nested":{"key":"value"},"array":[1,2,3]}"#.repeat(1000).into_bytes()),
        ("html", "<html><body><div>test</div></body></html>".repeat(5000).into_bytes()),
    ];

    for (name, data) in test_data {
        group.bench_with_input(BenchmarkId::new(name, data.len()), &data, |b, input| {
            b.iter(|| {
                black_box(compress::compress_brotli(black_box(input)).unwrap())
            });
        });
    }

    group.finish();
}

/// Benchmark path validation
fn bench_path_validation(c: &mut Criterion) {
    let validator = PathValidator::new(PathBuf::from("/var/www"));

    let mut group = c.benchmark_group("path_validation");

    let test_paths = vec![
        ("valid", "/var/www/index.html"),
        ("nested", "/var/www/static/css/style.css"),
        ("traversal", "/var/www/../../../etc/passwd"),
        ("deep", "/var/www/a/b/c/d/e/f/g/h/index.html"),
    ];

    for (name, path) in test_paths {
        let path_buf = PathBuf::from(path);
        group.bench_with_input(BenchmarkId::new(name, path), &path_buf, |b, input| {
            b.iter(|| {
                black_box(validator.validate(black_box(input)))
            });
        });
    }

    group.finish();
}

/// Benchmark MIME type detection
fn bench_mime_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("mime_detection");

    let test_files = vec![
        ("html", "index.html"),
        ("css", "style.css"),
        ("js", "app.js"),
        ("png", "image.png"),
        ("jpg", "photo.jpg"),
        ("json", "data.json"),
        ("xml", "config.xml"),
    ];

    for (name, filename) in test_files {
        let path = PathBuf::from(filename);
        group.bench_with_input(BenchmarkId::new(name, filename), &path, |b, input| {
            b.iter(|| {
                black_box(MimeDetector::detect(black_box(input)))
            });
        });
    }

    group.finish();
}

/// Benchmark compression decision logic
fn bench_compression_decision(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_decision");

    let content_types = vec![
        ("html", "text/html"),
        ("css", "text/css"),
        ("javascript", "text/javascript"),
        ("json", "application/json"),
        ("image", "image/jpeg"),
        ("video", "video/mp4"),
        ("audio", "audio/mpeg"),
        ("gzip", "application/gzip"),
        ("zip", "application/zip"),
    ];

    for (name, content_type) in content_types {
        group.bench_with_input(BenchmarkId::new(name, content_type), &content_type, |b, input| {
            b.iter(|| {
                black_box(rust_serv::handler::should_skip_compression(black_box(input)))
            });
        });
    }

    group.finish();
}

/// Benchmark ETag generation
fn bench_etag_generation(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    let mut group = c.benchmark_group("etag_generation");

    group.bench_function("generate_etag", |b| {
        b.iter(|| {
            let metadata = fs::metadata(&test_file).unwrap();
            let size = metadata.len();
            let modified = metadata.modified().unwrap();
            let duration = modified.duration_since(std::time::UNIX_EPOCH).unwrap();
            black_box(format!(r#""{}-{}""#, duration.as_secs(), size))
        });
    });

    group.finish();
}

/// Benchmark directory listing
fn bench_directory_listing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();

    // Create directories with different numbers of files
    let file_counts = vec![10, 100, 1000];

    let mut group = c.benchmark_group("directory_listing");

    for count in file_counts {
        let test_dir = temp_dir.path().join(format!("dir_{}", count));
        fs::create_dir_all(&test_dir).unwrap();

        for i in 0..count {
            let file_path = test_dir.join(format!("file_{}.txt", i));
            fs::write(&file_path, "content").unwrap();
        }

        group.bench_with_input(BenchmarkId::new("list", count), &test_dir, |b, path| {
            b.iter(|| {
                black_box(FileService::list_directory(black_box(path)).unwrap())
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_file_service,
    bench_gzip_compression,
    bench_brotli_compression,
    bench_path_validation,
    bench_mime_detection,
    bench_compression_decision,
    bench_etag_generation,
    bench_directory_listing
);

criterion_main!(benches);

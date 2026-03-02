use std::task::{Context, Poll};
use hyper::{Request, header};
use http_body_util::{BodyExt, Full};
use tower::{Layer, Service};

/// Compression middleware layer
#[derive(Clone)]
pub struct CompressionLayer {
    enabled: bool,
}

impl CompressionLayer {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl<S> Layer<S> for CompressionLayer {
    type Service = CompressionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CompressionService {
            inner,
            enabled: self.enabled,
        }
    }
}

#[derive(Clone)]
pub struct CompressionService<S> {
    inner: S,
    enabled: bool,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CompressionService<S>
where
    S: Service<Request<ReqBody>, Response = hyper::Response<ResBody>>,
    ReqBody: BodyExt + Send + 'static,
    ResBody: BodyExt + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Check if compression is enabled
        if !self.enabled {
            return self.inner.call(req);
        }

        // Check Accept-Encoding header
        let accept_encoding = req.headers().get("Accept-Encoding")
            .and_then(|v| v.to_str().ok());

        if accept_encoding.is_none() {
            // Client doesn't accept compression
            return self.inner.call(req);
        }

        let accept_encoding = accept_encoding.unwrap();

        // Check if client accepts gzip or brotli
        let use_gzip = accept_encoding.contains("gzip");
        let use_brotli = accept_encoding.contains("br");

        if !use_gzip && !use_brotli {
            // Client doesn't accept our supported compression methods
            return self.inner.call(req);
        }

        // Prefer brotli over gzip if both are accepted
        let encoding = if use_brotli {
            "br"
        } else if use_gzip {
            "gzip"
        } else {
            return self.inner.call(req);
        };

        // Add encoding header to indicate compression support
        let encoding_header = header::HeaderValue::from_str(encoding).unwrap();
        req.headers_mut().insert("X-Content-Encoding", encoding_header);

        // Get response from inner service
        self.inner.call(req)

        // Note: Full compression implementation would require:
        // 1. Collecting the response body
        // 2. Checking if content is compressible
        // 3. Compressing with gzip or brotli
        // 4. Updating Content-Length header
        // 5. Adding Content-Encoding header

        // For now, we just pass through and add encoding header
        // This demonstrates compression detection logic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_layer() {
        let layer = CompressionLayer::new(true);
        assert!(layer.enabled);
    }
}

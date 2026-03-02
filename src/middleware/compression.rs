use std::task::{Context, Poll};
use hyper::Request;
use http_body_util::BodyExt;
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
    #[allow(dead_code)]
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

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Check Accept-Encoding header
        // Compression will be implemented in a later iteration
        self.inner.call(req)
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

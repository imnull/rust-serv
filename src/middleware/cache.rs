use std::task::{Context, Poll};
use hyper::Request;
use http_body_util::BodyExt;
use tower::{Layer, Service};

/// Cache middleware layer
#[derive(Clone)]
pub struct CacheLayer;

impl<S> Layer<S> for CacheLayer {
    type Service = CacheService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService { inner }
    }
}

#[derive(Clone)]
pub struct CacheService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CacheService<S>
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
        // Check If-None-Match, If-Modified-Since headers
        // Caching will be implemented in a later iteration
        self.inner.call(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_layer() {
        // Test in integration tests
    }
}

use std::task::{Context, Poll};
use hyper::Request;
use http_body_util::BodyExt;
use tower::{Layer, Service};

/// Logging middleware layer
#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingService<S>
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
        tracing::info!("{} {}", req.method(), req.uri().path());
        self.inner.call(req)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_logging_layer() {
        // Test in integration tests
    }
}

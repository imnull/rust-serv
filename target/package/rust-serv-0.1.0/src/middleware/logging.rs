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
    use super::*;
    use hyper::{Request, Response, Method};
    use http_body_util::Full;
    use hyper::body::Bytes;
    use tower::{ServiceBuilder, ServiceExt};
    use std::pin::Pin;
    use std::future::Future;
    use std::task::{Context, Poll};

    /// Mock service for testing
    #[derive(Clone)]
    struct MockService;

    impl Service<Request<Full<Bytes>>> for MockService {
        type Response = Response<Full<Bytes>>;
        type Error = std::convert::Infallible;
        type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request<Full<Bytes>>) -> Self::Future {
            let method = req.method().clone();
            let uri = req.uri().clone();

            Box::pin(async move {
                Ok(Response::new(Full::new(Bytes::from(format!(
                    "Response to {} {}",
                    method,
                    uri.path()
                )))))
            })
        }
    }

    #[test]
    fn test_logging_layer_creation() {
        let _layer = LoggingLayer;
        // Layer should be created successfully
    }

    #[test]
    fn test_logging_layer_clone() {
        let layer = LoggingLayer;
        let _cloned = layer.clone();
        // Layer should be clonable
    }

    #[test]
    fn test_logging_service_creation() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let _logging_service = layer.layer(mock_service);
        // LoggingService should be created successfully
    }

    #[tokio::test]
    async fn test_logging_service_call() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let mut logging_service = layer.layer(mock_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = logging_service.call(request).await.unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logging_service_with_post() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let mut logging_service = layer.layer(mock_service);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/users")
            .body(Full::new(Bytes::from("test data")))
            .unwrap();

        let response = logging_service.call(request).await.unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logging_service_with_query_params() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let mut logging_service = layer.layer(mock_service);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/search?q=rust&limit=10")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let response = logging_service.call(request).await.unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logging_service_poll_ready() {
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

        let layer = LoggingLayer;
        let mock_service = MockService;
        let mut logging_service = layer.layer(mock_service);

        // Create a dummy waker
        fn dummy_clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        fn dummy(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(dummy_clone, dummy, dummy, dummy);
        let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        let poll_result = logging_service.poll_ready(&mut cx);
        assert!(matches!(poll_result, Poll::Ready(Ok(()))));
    }

    #[test]
    fn test_logging_service_clone() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let logging_service = layer.layer(mock_service);
        let _cloned = logging_service.clone();
        // LoggingService should be clonable
    }

    #[tokio::test]
    async fn test_logging_service_multiple_requests() {
        let layer = LoggingLayer;
        let mock_service = MockService;
        let mut logging_service = layer.layer(mock_service);

        // Make multiple requests
        for i in 0..5 {
            let request = Request::builder()
                .method(Method::GET)
                .uri(&format!("/page/{}", i))
                .body(Full::new(Bytes::new()))
                .unwrap();

            let response = logging_service.call(request).await.unwrap();
            assert_eq!(response.status(), hyper::StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_logging_service_with_different_methods() {
        let methods = vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ];

        for method in methods {
            let layer = LoggingLayer;
            let mock_service = MockService;
            let mut logging_service = layer.layer(mock_service);

            let request = Request::builder()
                .method(method.clone())
                .uri("/test")
                .body(Full::new(Bytes::new()))
                .unwrap();

            let response = logging_service.call(request).await.unwrap();
            assert_eq!(response.status(), hyper::StatusCode::OK);
        }
    }
}

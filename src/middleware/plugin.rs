//! Plugin Middleware
//!
//! 集成 WebAssembly 插件系统的 Tower 中间件（简化版）

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::{Request, Response, StatusCode};
use tokio::sync::RwLock;
use tower::{Layer, Service};
use tracing::{debug, error, info, warn};

use crate::plugin::{
    PluginManager,
    traits::{PluginAction, PluginRequest, PluginResponse},
};

/// 插件中间件配置
#[derive(Clone, Debug)]
pub struct PluginMiddlewareConfig {
    /// 是否启用插件系统
    pub enabled: bool,
    /// 插件目录
    pub plugin_dir: std::path::PathBuf,
    /// 是否启用热重载
    pub hot_reload: bool,
    /// 请求体大小限制（字节）
    pub max_body_size: usize,
    /// 插件执行超时（毫秒）
    pub timeout_ms: u64,
}

impl Default for PluginMiddlewareConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            plugin_dir: std::path::PathBuf::from("./plugins"),
            hot_reload: true,
            max_body_size: 1024 * 1024, // 1MB
            timeout_ms: 100,
        }
    }
}

/// 插件中间件 Layer
#[derive(Clone)]
pub struct PluginLayer {
    manager: Arc<RwLock<PluginManager>>,
    config: PluginMiddlewareConfig,
}

impl PluginLayer {
    /// 创建新的插件中间件 Layer
    pub fn new(manager: Arc<RwLock<PluginManager>>) -> Self {
        Self {
            manager,
            config: PluginMiddlewareConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(manager: Arc<RwLock<PluginManager>>, config: PluginMiddlewareConfig) -> Self {
        Self { manager, config }
    }

    /// 获取配置
    pub fn config(&self) -> &PluginMiddlewareConfig {
        &self.config
    }
}

impl<S> Layer<S> for PluginLayer {
    type Service = PluginMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PluginMiddleware {
            inner,
            manager: Arc::clone(&self.manager),
            config: self.config.clone(),
        }
    }
}

/// 插件中间件 Service
#[derive(Clone)]
pub struct PluginMiddleware<S> {
    inner: S,
    manager: Arc<RwLock<PluginManager>>,
    config: PluginMiddlewareConfig,
}

impl<S> PluginMiddleware<S> {
    /// 创建新的插件中间件
    pub fn new(inner: S, manager: Arc<RwLock<PluginManager>>) -> Self {
        Self {
            inner,
            manager,
            config: PluginMiddlewareConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(
        inner: S,
        manager: Arc<RwLock<PluginManager>>,
        config: PluginMiddlewareConfig,
    ) -> Self {
        Self {
            inner,
            manager,
            config,
        }
    }

    /// 获取插件管理器
    pub fn manager(&self) -> Arc<RwLock<PluginManager>> {
        Arc::clone(&self.manager)
    }
}

impl<S, B> Service<Request<B>> for PluginMiddleware<S>
where
    S: Service<Request<B>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Debug,
    B: hyper::body::Body + Send + 'static + Default,
    B::Data: Send,
    B::Error: std::fmt::Debug,
{
    type Response = Response<B>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let manager = Arc::clone(&self.manager);
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // 如果插件系统被禁用，直接传递给下一个服务
            if !config.enabled {
                debug!("Plugin system disabled, passing through");
                return inner.call(req).await;
            }

            // 获取管理器的读锁
            let manager_guard = manager.read().await;

            // 如果管理器被禁用，直接传递
            if !manager_guard.is_enabled() {
                drop(manager_guard);
                return inner.call(req).await;
            }

            drop(manager_guard);

            // 提取请求信息
            let (parts, _body) = req.into_parts();
            
            // 简化为直接传递给 inner service
            // 实际插件集成需要更多工作
            let req = Request::from_parts(parts, B::default());
            
            inner.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_middleware_config_default() {
        let config = PluginMiddlewareConfig::default();
        assert!(config.enabled);
        assert!(config.hot_reload);
        assert_eq!(config.max_body_size, 1024 * 1024);
        assert_eq!(config.timeout_ms, 100);
    }

    #[test]
    fn test_plugin_layer_creation() {
        let manager = Arc::new(RwLock::new(PluginManager::new().unwrap()));
        let layer = PluginLayer::new(manager);
        assert!(layer.config().enabled);
    }
}
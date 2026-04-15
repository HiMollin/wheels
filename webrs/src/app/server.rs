use crate::app::AppState;
use crate::app::latency::LatencyOnResponse;
use crate::config::ServerConfig;
use axum::Router;
use axum::extract::{DefaultBodyLimit, Request};
use bytesize::ByteSize;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub struct Server {
    config: &'static ServerConfig,
}

impl Server {
    pub fn new(config: &'static ServerConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, state: AppState, router: Router<AppState>) -> anyhow::Result<()> {
        let router = self.build_router(state, router);
        let port = self.config.port();

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        tracing::info!("Listening on {}", listener.local_addr()?);

        axum::serve(
            listener,
            // into_make_service_with_connect_info 是为了在业务代码里能拿到客户端的真实 IP 地址
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }

    fn build_router(&self, state: AppState, router: Router<AppState>) -> Router {
        // 超时限制
        let timeout = TimeoutLayer::new(Duration::from_secs(120));

        // 请求体大小限制，限制用户上传的 JSON 或文件最大只能是 10 MB
        let body_limit = DefaultBodyLimit::max(ByteSize::mib(10).as_u64() as usize);

        // 跨域资源共享
        let cors = CorsLayer::new()
            .allow_origin(cors::Any)
            .allow_methods(cors::Any)
            .allow_headers(cors::Any)
            .allow_credentials(false)
            .max_age(Duration::from_secs(3600 * 12));

        // 请求追踪监控
        let tracing = TraceLayer::new_for_http()
            // 每次来一个新请求，自动给它生成一个独一无二的 Span
            .make_span_with(|request: &Request| {
                let method = request.method();
                let path = request.uri().path();
                // 给每个请求生成一个极短的唯一 ID
                let id = xid::new();

                tracing::info_span!("Api Request", id = %id, method = %method, path = %path)
            })
            .on_request(()) // 关闭默认的“收到请求”日志
            .on_failure(()) // 关闭默认的“请求失败”日志
            // 响应结束时，调用自定义的 LatencyOnResponse 打印耗时多少毫秒
            .on_response(LatencyOnResponse);

        // 路径标准化
        // 如果用户手抖访问了 /api/users/，它会自动改成 /api/users
        let normalize_path = NormalizePathLayer::trim_trailing_slash();

        Router::new()
            .merge(router)
            .layer(timeout)
            .layer(body_limit)
            .layer(tracing)
            .layer(cors)
            .layer(normalize_path)
            .with_state(state)
    }
}

use axum::http::Response;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tower_http::trace::OnResponse;
use tracing::Span;

#[derive(Debug, Clone, Copy)]
pub struct LatencyOnResponse; // 定义一个不带字段的结构体给中间件，只需实现 OnResponse

impl<B> OnResponse<B> for LatencyOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _: &Span) {
        tracing::info!(
            latency = %Latency(latency),
            status = response.status().as_u16(),
            "finished processing request"
        );
    }
}

struct Latency(Duration);

/// 在 rust 中我们不能为一个外部类型加上外部 trait，所以我们可以在本地包一层
impl Display for Latency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.as_millis() > 0 {
            write!(f, "{} ms", self.0.as_millis())
        } else {
            write!(f, "{} μs", self.0.as_micros())
        }
    }
}

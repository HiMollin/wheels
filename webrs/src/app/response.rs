use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn new(code: i32, message: String, data: Option<T>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    pub fn ok<M: AsRef<str>>(message: M, data: Option<T>) -> Self {
        Self::new(0, message.as_ref().to_string(), data)
    }

    pub fn err<M: AsRef<str>>(message: M) -> Self {
        Self::new(1, message.as_ref().to_string(), None)
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

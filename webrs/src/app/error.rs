use axum::{
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_valid::ValidRejection;

use crate::app::response::ApiResponse;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("服务器迷路了")]
    NotFound,
    #[error("请求方法不支持")]
    MethodNotAllowed,
    #[error("数据库异常: {0}")]
    Database(#[from] sqlx::Error),

    // Axum 提取参数（URL查询参数、路径参数、JSON体）失败时自动转化的错误
    #[error("查询参数错误: {0}")]
    Query(#[from] QueryRejection),
    #[error("路径参数错误: {0}")]
    Path(#[from] PathRejection),
    #[error("Body参数错误: {0}")]
    Json(#[from] JsonRejection),

    // 表单/字段验证失败（比如邮箱格式不对），包含具体的报错信息字符串
    #[error("参数校验失败: {0}")]
    Validation(String),

    // 密码加密/解密报错时自动转换
    #[error("密码Hash错误: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("JWT错误: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("未授权: {0}")]
    Unauthenticated(String),
    #[error("{0}")]
    Biz(String),

    // 其他所有不知道怎么分类的杂七杂八的错，统统扔给 anyhow 兜底
    #[error("错误: {0}")]
    Internal(#[from] anyhow::Error),
}

/// ValidRejection 本身是一个拥有两个分支的枚举，无法直接 #[from]
impl From<axum_valid::ValidRejection<ApiError>> for ApiError {
    fn from(value: ValidRejection<ApiError>) -> Self {
        match value {
            ValidRejection::Valid(errors) => ApiError::Validation(errors.to_string()),
            ValidRejection::Inner(error) => error,
        }
    }
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND, // 404
            ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED, // 405

            // 数据库炸了、密码库炸了、未知兜底错 -> 统统算服务器背锅 500
            ApiError::Database(_) | ApiError::Bcrypt(_) | ApiError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }

            // 传参不对、JSON格式不对、校验不通过 -> 统统算前端背锅 400
            ApiError::Query(_)
            | ApiError::Path(_)
            | ApiError::Json(_)
            | ApiError::Validation(_) => StatusCode::BAD_REQUEST,

            // 没登录、Token不对， 401 让他去登录
            ApiError::JWT(_) | ApiError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,

            // 业务错误，HTTP状态码依然给 200 OK，
            // 只是在 JSON body 里通过自定义的错误码来告诉前端具体发生了什么。
            ApiError::Biz(_) => StatusCode::OK,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let body = axum::Json(ApiResponse::<()>::err(self.to_string()));

        (status_code, body).into_response()
    }
}

impl From<ApiError> for Response {
    fn from(value: ApiError) -> Self {
        value.into_response()
    }
}

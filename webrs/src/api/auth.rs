use crate::app::AppState;
use crate::app::auth::{Principal, get_jwt};
use crate::app::error::{ApiError, ApiResult};
use crate::app::middleware::get_auth_layer;
use crate::app::response::ApiResponse;
use crate::app::utils::verify_password;
use crate::app::valid::ValidJson;
use crate::entity::sys_user;
use axum::extract::{ConnectInfo, State};
use axum::{Extension, Router, debug_handler, routing};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

pub fn create_router() -> Router<AppState> {
    Router::new()
        // 获取用户信息的接口
        .route("/user-info", routing::get(get_user_info))
        .route_layer(get_auth_layer())
        // 注册登录接口
        .route("/login", routing::post(login))
}

/// 登录请求结构体
#[derive(Debug, Deserialize, Validate)]
pub struct LoginParams {
    #[validate(length(min = 3, max = 16, message = "账号长度为3-16"))]
    account: String, // 限制账号长度
    #[validate(length(min = 6, max = 16, message = "密码长度为6-16"))]
    password: String, // 限制密码长度
}

/// 登录成功的返回结构
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    access_token: String,
}

#[debug_handler]
// 自动记录进入这个函数时的账号和 IP，且跳过记录的长参数(skip_all)
#[tracing::instrument(name = "login", skip_all, fields(account = %params.account, ip = %addr.ip()))]
async fn login(
    State(AppState { db }): State<AppState>,
    // 拿到用户的真实 IP 地址
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ValidJson(params): ValidJson<LoginParams>,
) -> ApiResult<ApiResponse<LoginResult>> {
    tracing::info!("开始处理登录逻辑...");

    // 找账号
    let user = sqlx::query_as::<_, sys_user::Model>(
        "select id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at \
         from sys_user where account = $1",
    )
        .bind(&params.account)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| ApiError::Biz(String::from("账号或密码不正确")))?;

    // 比对密码
    if !verify_password(&params.password, &user.password)? {
        return Err(ApiError::Biz(String::from("账号或密码不正确")));
    }

    // 制发 JWT
    let principal = Principal {
        id: user.id,
        name: user.name,
    };
    let access_token = get_jwt().encode(principal)?;

    tracing::info!("登录成功，JWT Token: {}", access_token);

    // 包装送给前端
    Ok(ApiResponse::ok(
        "登录成功",
        Some(LoginResult { access_token }),
    ))
}

// 获取当前登录用户信息的接口
#[debug_handler]
async fn get_user_info(
    // Extension 提取器，把 JWTAuth 中间件存的 Principal 拿出来
    Extension(principal): Extension<Principal>,
) -> ApiResult<ApiResponse<Principal>> {
    Ok(ApiResponse::ok("ok", Some(principal)))
}

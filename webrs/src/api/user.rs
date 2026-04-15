use crate::app::AppState;
use crate::app::common::{Page, PaginationParams};
use crate::app::enumeration::Gender;
use crate::app::error::{ApiError, ApiResult};
use crate::app::id::next_id;
use crate::app::path::Path;
use crate::app::response::ApiResponse;
use crate::app::utils::encode_password;
use crate::app::valid::{ValidJson, ValidQuery};
use crate::entity::sys_user;
use axum::extract::State;
use axum::{Router, debug_handler, routing};
use chrono::NaiveDate;
use serde::Deserialize;
use validator::Validate;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(find_page))
        .route("/", routing::post(create))
        .route("/{id}", routing::put(update))
        .route("/{id}", routing::delete(delete))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserQueryParams {
    keyword: Option<String>,
    #[validate(nested)] // 嵌套校验分页参数
    #[serde(flatten)] // 扁平化展开
    pagination: PaginationParams,
}

#[debug_handler]
async fn find_page(
    State(AppState { db }): State<AppState>,
    ValidQuery(UserQueryParams {
        keyword,
        pagination,
    }): ValidQuery<UserQueryParams>,
) -> ApiResult<ApiResponse<Page<sys_user::Model>>> {
    let limit = pagination.size as i64;
    let offset = ((pagination.page - 1) * pagination.size) as i64;

    let total = if let Some(keyword) = keyword.as_deref() {
        let pattern = format!("%{}%", keyword);
        sqlx::query_scalar::<_, i64>(
            "select count(*) from sys_user where name ilike $1 or account ilike $1",
        )
        .bind(pattern)
        .fetch_one(&db)
        .await? as u64
    } else {
        sqlx::query_scalar::<_, i64>("select count(*) from sys_user")
            .fetch_one(&db)
            .await? as u64
    };

    let items = if let Some(keyword) = keyword.as_deref() {
        let pattern = format!("%{}%", keyword);
        sqlx::query_as::<_, sys_user::Model>(
            "select id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at \
             from sys_user \
             where name ilike $1 or account ilike $1 \
             order by created_at desc \
             limit $2 offset $3",
        )
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&db)
        .await?
    } else {
        sqlx::query_as::<_, sys_user::Model>(
            "select id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at \
             from sys_user \
             order by created_at desc \
             limit $1 offset $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&db)
        .await?
    };

    // 打包成标准 Page 对象
    let page = Page::from_pagination(pagination, total, items);

    Ok(ApiResponse::ok("ok", Some(page)))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserParams {
    #[validate(length(min = 1, max = 16, message = "姓名长度为1-16"))]
    pub name: String,
    pub gender: Gender, // 前端传男/女，全自动变枚举
    #[validate(length(min = 1, max = 16, message = "账号长度为1-16"))]
    pub account: String,
    #[validate(length(min = 6, max = 16, message = "密码长度为6-16"))]
    pub password: String,
    #[validate(custom(function = "crate::app::validation::is_mobile_phone"))]
    pub mobile_phone: String,
    pub birthday: NaiveDate,
    #[serde(default)]
    pub enabled: bool,
}

#[debug_handler]
async fn create(
    State(AppState { db }): State<AppState>,
    ValidJson(params): ValidJson<UserParams>,
) -> ApiResult<ApiResponse<sys_user::Model>> {
    if params.password.is_empty() {
        return Err(ApiError::Biz("密码不能为空".to_string()));
    }

    let result = sqlx::query_as::<_, sys_user::Model>(
        "insert into sys_user (id, name, gender, account, password, mobile_phone, birthday, enabled) \
         values ($1, $2, $3, $4, $5, $6, $7, $8) \
         returning id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at",
    )
    .bind(next_id())
    .bind(&params.name)
    .bind(params.gender)
    .bind(&params.account)
    .bind(encode_password(&params.password)?)
    .bind(&params.mobile_phone)
    .bind(params.birthday)
    .bind(params.enabled)
    .fetch_one(&db)
    .await?;

    Ok(ApiResponse::ok("ok", Some(result)))
}

#[debug_handler]
async fn update(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>,                   // 从 URL /123 拿到 ID
    ValidJson(params): ValidJson<UserParams>, // 从 Body 拿到修改后的数据
) -> ApiResult<ApiResponse<sys_user::Model>> {
    let existed_user = sqlx::query_as::<_, sys_user::Model>(
        "select id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at \
         from sys_user where id = $1",
    )
    .bind(&id)
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| ApiError::Biz(String::from("待修改的用户不存在")))?;

    let password = if params.password.is_empty() {
        existed_user.password
    } else {
        encode_password(&params.password)?
    };

    let result = sqlx::query_as::<_, sys_user::Model>(
        "update sys_user \
         set name = $1, gender = $2, account = $3, password = $4, mobile_phone = $5, birthday = $6, enabled = $7, updated_at = current_timestamp \
         where id = $8 \
         returning id, name, gender, account, password, mobile_phone, birthday, enabled, created_at, updated_at",
    )
    .bind(&params.name)
    .bind(params.gender)
    .bind(&params.account)
    .bind(password)
    .bind(&params.mobile_phone)
    .bind(params.birthday)
    .bind(params.enabled)
    .bind(&id)
    .fetch_one(&db)
    .await?;

    Ok(ApiResponse::ok("ok", Some(result)))
}

#[debug_handler]
async fn delete(
    State(AppState { db }): State<AppState>,
    Path(id): Path<String>, // 安全提取 ID
) -> ApiResult<ApiResponse<()>> {
    let existed_user = sqlx::query_scalar::<_, String>("select id from sys_user where id = $1")
        .bind(&id)
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| ApiError::Biz(String::from("待删除的用户不存在")))?;

    let result = sqlx::query("delete from sys_user where id = $1")
        .bind(existed_user)
        .execute(&db)
        .await?;

    // 打个日志：某某用户被删除了
    tracing::info!(
        "Deleted user: {}, affected rows: {}",
        id,
        result.rows_affected()
    );

    Ok(ApiResponse::ok("ok", None))
}

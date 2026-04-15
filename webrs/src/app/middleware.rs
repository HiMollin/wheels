use crate::app::auth::{JWT, get_jwt};
use crate::app::error::ApiError;
use axum::body::Body;
use axum::http::{Request, Response, header};
use std::pin::Pin;
use std::sync::LazyLock;
use tower_http::auth::{AsyncAuthorizeRequest, AsyncRequireAuthorizationLayer};

/// 声明一个全局静态变量 AUTH_LAYER，它是一个被懒加载锁包围的鉴权层。
static AUTH_LAYER: LazyLock<AsyncRequireAuthorizationLayer<JWTAuth>> =
    LazyLock::new(|| AsyncRequireAuthorizationLayer::new(JWTAuth::new(get_jwt())));

#[derive(Clone)]
pub struct JWTAuth {
    jwt: &'static JWT,
}

impl JWTAuth {
    pub fn new(jwt: &'static JWT) -> Self {
        Self { jwt }
    }
}

// 为 JWTAuth 实现 Tower 规定的异步鉴权特征
impl AsyncAuthorizeRequest<Body> for JWTAuth {
    type ResponseBody = Body;
    type RequestBody = Body;
    // 在 trait 里直接写 async fn 返回异步结果比较麻烦（涉及底层编译尺寸问题）
    // 所以必须用 Box 把 Future 包装到堆内存里，用 Pin 钉住不让它在内存里乱跑，并加上 Send 保证它可以跨线程传递
    type Future = Pin<
        Box<
            dyn Future<Output = Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>
                + Send
                + 'static,
        >,
    >;

    fn authorize(&mut self, mut request: Request<Body>) -> Self::Future {
        let jwt = self.jwt;

        Box::pin(async move {
            let token = request
                .headers()
                // 去请求的头部 (Headers) 找 "Authorization" 标签
                .get(header::AUTHORIZATION)
                .map(|value| -> Result<_, ApiError> {
                    let token = value
                        .to_str()
                        .map_err(|_| {
                            ApiError::Unauthenticated(String::from(
                                "Authorization请求头不是一个有效的字符串",
                            ))
                        })?
                        //按照国际惯例，Token 必须以 "Bearer " 开头，把这前缀切掉
                        .strip_prefix("Bearer ")
                        .ok_or_else(|| {
                            ApiError::Unauthenticated(String::from(
                                "Authorization请求头必须以 Bearer 开头",
                            ))
                        })?;

                    Ok(token)
                })
                // 此时外面的类型是 Option<Result<T, E>>
                // transpose() 会把它翻转成 Result<Option<T>, E>
                // 然后加上 ?，如果是 Error 就直接阻断返回给前端，如果是 Ok(Option)，就剥掉一层壳
                .transpose()?
                .ok_or_else(|| {
                    ApiError::Unauthenticated(String::from("Authorization请求头必须存在"))
                })?;
            // 拿到干净的 Token 后，塞进 jwt.decode() 里去验证签名和过期时间
            let principal = jwt.decode(token).map_err(|err| ApiError::Internal(err))?;
            // 把解密出来的用户信息，塞进 request 的 extensions 里
            // 这样排在后面的业务 Handler 就能直接知道是谁在操作了
            request.extensions_mut().insert(principal);

            Ok(request)
        })
    }
}

pub fn get_auth_layer() -> &'static AsyncRequireAuthorizationLayer<JWTAuth> {
    &AUTH_LAYER
}

use crate::app::error::ApiResult;

// 注册或修改密码，将密码加密
pub fn encode_password(password: &str) -> ApiResult<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

// 验证密码，把用户输入的和加密过的比对
pub fn verify_password(password: &str, hashed_password: &str) -> ApiResult<bool> {
    Ok(bcrypt::verify(password, hashed_password)?)
}

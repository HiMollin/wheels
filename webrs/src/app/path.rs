use axum::extract::FromRequestParts;
use axum_valid::HasValidate;

use crate::app::error::ApiError;

#[derive(Debug, Clone, Default, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T); // 将错误类型统一

impl<T> HasValidate for Path<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

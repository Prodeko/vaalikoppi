use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    api_types::{ApiError, ApiResult},
    models::LoginState,
};

#[derive(Clone, Debug)]
pub struct Ctx {
    pub login_state: LoginState,
}

impl Ctx {
    pub fn new(login_state: LoginState) -> Self {
        Self { login_state }
    }

    pub fn login_state(&self) -> LoginState {
        self.login_state.clone()
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> ApiResult<Self> {
        parts
            .extensions
            .get::<Ctx>()
            .map(|voting| voting.clone())
            .ok_or(ApiError::InternalServerError)
    }
}

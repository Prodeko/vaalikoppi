use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    api_types::{ApiError, ApiResult, AuthFailedError::MissingToken},
    ctx::Ctx,
    models::{ElectionId, LoginState},
};

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ElectionId {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> ApiResult<Self> {
        let ctx = parts
            .extensions
            .get::<Ctx>()
            .ok_or_else(|| ApiError::AuthFailed(MissingToken))?;

        match &ctx.login_state {
            LoginState::NotLoggedIn => Err(ApiError::AuthFailed(MissingToken)),
            LoginState::Voter { election_id, .. } => Ok(*election_id),
            LoginState::Admin { election_id, .. } => Ok(*election_id),
        }
    }
}

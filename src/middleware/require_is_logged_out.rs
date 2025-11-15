use crate::{
    api_types::{ApiError, ApiResult, AuthFailedError::TokenNotExpected},
    ctx::Ctx,
};
use axum::{http::Request, middleware::Next, response::Response};

pub async fn require_is_logged_out<B>(
    context: Ctx,
    req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    let state = context.login_state();

    match state {
        crate::models::LoginState::NotLoggedIn => Ok(next.run(req).await),
        _ => Err(ApiError::AuthFailed(TokenNotExpected)),
    }
}

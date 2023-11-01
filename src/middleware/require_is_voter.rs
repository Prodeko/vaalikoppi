use crate::{
    api_types::{ApiError, ApiResult},
    ctx::Ctx,
};
use axum::{http::Request, middleware::Next, response::Response};

pub async fn require_is_voter<B>(
    context: Ctx,
    req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    println!("{:?}", context);
    let state = context.login_state();

    match state {
        crate::models::LoginState::Voter { .. } => Ok(next.run(req).await),
        _ => Err(ApiError::TokenNotFound),
    }
}

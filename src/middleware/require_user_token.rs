use crate::{
    api_types::{
        ApiError::AuthFailed,
        ApiResult,
        AuthFailedError::{InvalidToken, MissingToken},
    },
    ctx::Ctx,
    models::{Token, TokenState},
};
use axum::{http::Request, middleware::Next, response::Response};

pub async fn require_user_token<B>(
    context: Ctx,
    req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    println!("{:?}", context);
    let token = context.token();

    return match token {
        Some(Token {
            state: TokenState::Activated,
            ..
        }) => Ok(next.run(req).await),
        _ => Err(AuthFailed(MissingToken)),
    };
}

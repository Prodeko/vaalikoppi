use crate::{
    ctx::Ctx,
    error::{
        AuthFailedError::{InvalidToken, MissingToken},
        Error::AuthFailed,
        Result,
    },
    models::{Token, TokenState},
};
use axum::{http::Request, middleware::Next, response::Response};

pub async fn require_user_token<B>(
    context: Ctx,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
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

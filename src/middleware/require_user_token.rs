use crate::{
    ctx::Ctx,
    error::{
        AuthFailedError::{InvalidToken, MissingToken},
        Error::AuthFailed,
        Result,
    },
    models::Token,
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
        None => Err(AuthFailed(MissingToken)),
        Some(Token {
            is_activated: false,
            ..
        }) => Err(AuthFailed(InvalidToken)),
        Some(Token {
            is_trashed: true, ..
        }) => Err(AuthFailed(InvalidToken)),
        Some(Token {
            is_activated: true,
            is_trashed: false,
            ..
        }) => Ok(next.run(req).await),
    };
}

use crate::{
    ctx::Ctx,
    error::{
        AuthFailedError::{InvalidToken, MissingToken},
        Error::{self, AuthFailed},
        Result,
    },
    http::{
        login::{JsonWebTokenClaims, AUTH_TOKEN},
        AppState,
    },
};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use tower_cookies::Cookies;

pub async fn require_admin<B>(context: Ctx, req: Request<B>, next: Next<B>) -> Result<Response> {
    println!("{:?}", context);

    if !context.is_admin() {
        return Err(AuthFailed(InvalidToken));
    }
    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<Ctx>()
            .map(|ctx| ctx.clone())
            .ok_or(AuthFailed(InvalidToken))
    }
}

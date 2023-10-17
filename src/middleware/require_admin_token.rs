use crate::{
    ctx::Ctx,
    error::{
        AuthFailedError::{InvalidToken, MissingToken},
        Error::{self, AuthFailed},
        Result,
    },
    http::{
        admin::login::{JsonWebTokenClaims, AUTH_TOKEN},
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

pub async fn resolve_ctx<B>(
    cookies: Cookies,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    let admin_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let decoded_token: Result<TokenData<JsonWebTokenClaims>> = admin_token.map_or_else(
        || Err(AuthFailed(MissingToken)),
        |t| {
            decode::<JsonWebTokenClaims>(
                &t,
                &DecodingKey::from_secret(state.config.hmac_key.as_bytes()),
                &Validation::default(),
            )
            .map_err(|_| AuthFailed(InvalidToken))
        },
    );

    let ctx: Ctx = Ctx::new(decoded_token.is_ok());

    req.extensions_mut().insert(ctx);

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

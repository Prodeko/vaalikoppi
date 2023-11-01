use crate::{
    api_types::{
        ApiError::AuthFailed,
        ApiResult,
        AuthFailedError::{InvalidToken, MissingToken},
    },
    ctx::Ctx,
    http::{
        login::{JsonWebTokenClaims, AUTH_TOKEN},
        user::USER_TOKEN,
        AppState,
    },
    models::{Token, TokenState},
};
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use tower_cookies::Cookies;

pub async fn resolve_ctx<B>(
    cookies: Cookies,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    let admin_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    let user_token = cookies.get(USER_TOKEN).map(|c| c.value().to_string());

    let resolved_admin_token: ApiResult<TokenData<JsonWebTokenClaims>> = admin_token.map_or_else(
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

    let resolved_user_token = sqlx::query_as!(
        Token,
        "
        SELECT
            id,
            token,
            state AS \"state: TokenState\",
            alias
        FROM token
        WHERE token = $1
        ",
        user_token
    )
    .fetch_optional(&state.db)
    .await?;

    let ctx: Ctx = Ctx::new(resolved_admin_token.is_ok(), resolved_user_token);

    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}

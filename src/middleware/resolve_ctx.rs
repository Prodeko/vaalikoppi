use crate::{
    api_types::ApiResult,
    ctx::Ctx,
    http::{
        login::{JsonWebTokenClaims, AUTH_TOKEN},
        user::VOTER_TOKEN,
        AppState,
    },
    models::{LoginState, Token, TokenState},
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
    let voter_token = cookies.get(VOTER_TOKEN).map(|c| c.value().to_string());

    // Check if valid voter token is found.
    // Default to LoginState::Voter instead of LoginState::Admin if both are found
    // because lost voter tokens often lead to voiding of all active tokens
    let resolved_voter_token = match voter_token {
        Some(token) => {
            sqlx::query_as!(
                Token,
                "
                SELECT
                id,
                election_id,
                token,
                state AS \"state: TokenState\",
                alias
                FROM token
                WHERE token = $1
                ",
                token
            )
            .fetch_optional(&state.db)
            .await
        }
        None => Ok(None),
    }?;

    if let Some(token) = resolved_voter_token {
        match (token.state, token.alias) {
            (TokenState::Activated, Some(alias)) => {
                let ctx = Ctx::new(LoginState::Voter {
                    token: token.token,
                    alias,
                    election_id: token.election_id,
                });
                req.extensions_mut().insert(ctx);
                return Ok(next.run(req).await);
            }
            _ => (),
        }
    }

    let admin_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    // Check if valid admin token is found
    let resolved_admin_token: Option<TokenData<JsonWebTokenClaims>> = admin_token
        .map(|t| {
            decode::<JsonWebTokenClaims>(
                &t,
                &DecodingKey::from_secret(state.config.hmac_key.as_bytes()),
                &Validation::default(),
            )
            .ok()
        })
        .flatten();

    if let Some(token_data) = resolved_admin_token {
        let ctx = Ctx::new(LoginState::Admin {
            election_id: token_data.claims.election_id,
        });
        req.extensions_mut().insert(ctx);
        return Ok(next.run(req).await);
    }

    let ctx: Ctx = Ctx::new(LoginState::NotLoggedIn);
    req.extensions_mut().insert(ctx);
    Ok(next.run(req).await)
}

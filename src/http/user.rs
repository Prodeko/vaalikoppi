use crate::models::{Token, TokenState};
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use sqlx::{self, Pool};
use tower_cookies::{Cookie, Cookies};

use crate::{
    api_types::{ApiError, ApiResult},
    http::AppState,
};

pub const USER_TOKEN: &str = "user-token";
pub const USER_TOKEN_MAX_AGE_DAYS: i64 = 1;

#[derive(Deserialize)]
struct LoginPayload {
    alias: String,
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {}

pub fn router() -> Router<AppState> {
    Router::new().route("/login/", post(user_login))
}

async fn user_login(
    state: State<AppState>,
    cookies: Cookies,
    Json(login_payload): Json<LoginPayload>,
) -> ApiResult<Json<LoginResponse>> {
    let row = sqlx::query_as!(
        Token,
        "
        SELECT
            id,
            token,
            state AS \"state: TokenState\",
            alias
        FROM token
        WHERE token = $1;
        ",
        login_payload.token
    )
    .fetch_one(&state.db)
    .await?;

    if row.state != TokenState::Activated {
        return Err(ApiError::LoginFail);
    }

    // register alias
    let token =
        register_and_validate_alias(&state.0.db, &login_payload.token, &login_payload.alias)
            .await?;

    cookies.add(
        Cookie::build(USER_TOKEN, token.token)
            .http_only(true)
            .path("/")
            .secure(true)
            .max_age(time::Duration::days(USER_TOKEN_MAX_AGE_DAYS))
            .finish(),
    );
    return Ok(Json(LoginResponse {}));
}

async fn register_and_validate_alias(
    executor: &Pool<Postgres>,
    token: &str,
    alias: &str,
) -> ApiResult<Token> {
    if alias.len() < 4 || alias.len() > 16 {
        return Err(ApiError::InvalidInput);
    }

    // Database should check for alias uniqueness
    let token = sqlx::query_as!(
        Token,
        "
        UPDATE token
        SET alias = $1
        WHERE token = $2
        RETURNING
            id,
            token,
            state AS \"state: TokenState\",
            alias
        ",
        alias,
        token
    )
    .fetch_one(executor)
    .await?;

    return Ok(token);
}

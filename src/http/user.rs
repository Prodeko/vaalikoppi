use std::time::Duration;

use crate::api_types::ApiError::{self, *};
use crate::api_types::AuthFailedError::{self};
use crate::api_types::InvalidAliasError::*;
use crate::models::{Token, TokenState};
use crate::{api_types::ApiResult, http::AppState};
use axum::error_handling::HandleErrorLayer;
use axum::{extract::State, routing::post, Json, Router};
use axum::{BoxError, Form};
use serde::{Deserialize, Serialize};
use sqlx::error::ErrorKind;
use sqlx::Postgres;
use sqlx::{self, Pool};
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_cookies::{Cookie, Cookies};

pub const VOTER_TOKEN: &str = "voter-token";
pub const VOTER_TOKEN_MAX_AGE_DAYS: i64 = 1;

#[derive(Deserialize)]
struct LoginPayload {
    alias: String,
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login/", post(user_login))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    println!("{}", err);
                    Err::<(), ApiError>(ApiError::InternalServerError)
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(100, Duration::from_secs(1))),
        )
        .route("/logout/", post(user_logout))
}

async fn user_login(
    state: State<AppState>,
    cookies: Cookies,
    Form(login_payload): Form<LoginPayload>,
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
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AuthFailed(AuthFailedError::MissingToken),
        _ => InternalServerError,
    })?;

    match row.state {
        TokenState::Unactivated => {
            return Err(ApiError::AuthFailed(AuthFailedError::TokenUnactivated))
        }
        TokenState::Voided => return Err(ApiError::AuthFailed(AuthFailedError::TokenVoided)),
        TokenState::Activated => {
            // register alias
            let token = register_and_validate_alias(
                &state.0.db,
                &login_payload.token,
                &login_payload.alias,
            )
            .await?;

            cookies.add(
                Cookie::build(VOTER_TOKEN, token.token)
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .max_age(time::Duration::days(VOTER_TOKEN_MAX_AGE_DAYS))
                    .finish(),
            );
            Ok(Json(LoginResponse {}))
        }
    }
}

#[derive(Serialize)]
struct UserLogoutResponse {
    status: i32,
}
async fn user_logout(cookies: Cookies) -> ApiResult<Json<UserLogoutResponse>> {
    let mut cookie = Cookie::named(VOTER_TOKEN);
    cookie.set_path("/");

    cookies.remove(cookie);
    Ok(Json(UserLogoutResponse { status: 0 }))
}

async fn register_and_validate_alias(
    executor: &Pool<Postgres>,
    token: &str,
    alias: &str,
) -> ApiResult<Token> {
    if alias.len() < 4 || alias.len() > 16 {
        return Err(ApiError::InvalidAlias(BadAlias));
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
    .await
    .map_err(|e| match e {
        // Handle unique key error (alias in use)
        sqlx::Error::Database(err) if err.kind() == ErrorKind::UniqueViolation => {
            InvalidAlias(AliasAlreadyInUse)
        }
        _ => InternalServerError,
    })?;

    return Ok(token);
}

use axum::{debug_handler, extract::State, routing::post, Json, Router};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};

use crate::{
    api_types::{ApiError, ApiResult},
    http::AppState,
};

pub const AUTH_TOKEN: &str = "admin-token";
const TOKEN_EXPIRY_DURATION_HOURS: i64 = 24;

#[derive(Deserialize)]
struct LoginPayload {
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonWebTokenClaims {
    exp: i64,
    iat: i64,
}

#[derive(Serialize)]
struct LoginResponse {}

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(json_web_token_login))
}

#[debug_handler]
async fn json_web_token_login(
    state: State<AppState>,
    cookies: Cookies,
    Json(login_payload): Json<LoginPayload>,
) -> ApiResult<Json<LoginResponse>> {
    if login_payload.token != state.config.admin_password {
        return Err(ApiError::LoginFail);
    }

    let current_timestamp = Utc::now();
    let expiration_time = current_timestamp + Duration::hours(TOKEN_EXPIRY_DURATION_HOURS);

    let claims = JsonWebTokenClaims {
        exp: expiration_time.timestamp(),
        iat: current_timestamp.timestamp(),
    };

    let token_result = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.hmac_key.as_bytes()),
    );

    token_result
        .map(|token| {
            cookies.add(
                Cookie::build(AUTH_TOKEN, token.clone())
                    .http_only(true)
                    .secure(true)
                    .expires(OffsetDateTime::from_unix_timestamp(claims.exp).unwrap()) // TODO: Fix nasty conversion between datetime types
                    .finish(),
            );
            Json(LoginResponse {})
        })
        .map_err(|_| ApiError::LoginFail)
}

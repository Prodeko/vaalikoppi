use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::http::{
    error::{Error, Result},
    AppState,
};

const TOKEN_EXPIRY_DURATION_HOURS: i64 = 24;

#[derive(Deserialize)]
struct LoginPayload {
    token: String,
}

#[derive(Serialize)]
struct JsonWebTokenClaims {
    exp: i64,
    iat: i64,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    exp: i64,
}

fn json_web_token_login(
    state: State<AppState>,
    Json(login_payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>> {
    if login_payload.token != state.config.admin_password {
        return Err(Error::LoginFail);
    }

    let current_timestamp = Utc::now();
    let claims = JsonWebTokenClaims {
        exp: (current_timestamp + Duration::hours(TOKEN_EXPIRY_DURATION_HOURS)).timestamp(),
        iat: current_timestamp.timestamp(),
    };

    let token_result = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.hmac_key.as_bytes()),
    );

    token_result
        .map(|token| {
            Json(LoginResponse {
                token,
                exp: claims.exp,
            })
        })
        .map_err(|_| Error::LoginFail)
}

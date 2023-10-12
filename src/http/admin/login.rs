use axum::extract::State;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::http::{
    error::{Error, Result},
    AppState,
};

const TOKEN_EXPIRY_DURATION: i64 = 24;

#[derive(Deserialize)]
struct LoginPayload {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct JsonWebTokenClaims {
    exp: i64,
    iat: i64,
}

fn json_web_token_login(state: State<AppState>, login_token: LoginPayload) -> Result<String> {
    if login_token.token == state.config.admin_password {
        let current_timestamp = Utc::now();
        let claims = JsonWebTokenClaims {
            exp: (current_timestamp + Duration::hours(TOKEN_EXPIRY_DURATION)).timestamp(),
            iat: current_timestamp.timestamp(),
        };

        return encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.config.hmac_key.as_bytes()),
        )
        .map_err(|_| Error::LoginFail);
    } else {
        Err(Error::LoginFail)
    }
}

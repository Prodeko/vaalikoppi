use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx;
use tower_cookies::{Cookie, Cookies};

use crate::{
    error::{Error, Result},
    http::AppState,
};

pub const AUTH_TOKEN: &str = "user-token";

#[derive(Deserialize)]
struct LoginPayload {
    alias: String,
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {}

pub fn router() -> Router<AppState> {
    Router::new().route("/vaalikoppi/user/login/", post(user_login))
}

async fn user_login(
    state: State<AppState>,
    cookies: Cookies,
    Json(login_payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>> {
    let result = sqlx::query!(
        "
        SELECT * FROM token WHERE id = $1
        ",
        login_payload.token.clone()
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => {
            if !row.is_activated || row.is_trashed {
                return Err(Error::LoginFail);
            } else {
                cookies.add(
                    Cookie::build(AUTH_TOKEN, login_payload.token)
                        .http_only(true)
                        .secure(true)
                        .finish(),
                );
                return Ok(Json(LoginResponse {}));
            }
        }
        Err(_err) => return Err(Error::LoginFail),
    }
}

use axum::middleware::from_fn;
use axum::routing::get;
use axum::{Form, Router};
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};

use crate::api_types::{ApiError, AuthFailedError};

use crate::http::login::{
    JsonWebTokenClaims, LoginResponse, AUTH_TOKEN, TOKEN_EXPIRY_DURATION_HOURS,
};
use crate::middleware::require_is_logged_out::require_is_logged_out;
use crate::models::LoginState;
use crate::{api_types::ApiResult, ctx::Ctx, http::AppState};
use askama::Template;
use axum::response::{Html, Redirect};
use axum::{debug_handler, extract::State, routing::post, Json};
use serde::{Deserialize, Serialize};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(post_create_election))
        .route("/", get(get_create_election))
        .route_layer(from_fn(require_is_logged_out))
}

#[derive(Template)]
#[template(path = "pages/create-election.html")]
pub struct CreateElectionTemplate {
    login_state: LoginState,
}

async fn get_create_election(ctx: Ctx) -> ApiResult<Html<String>> {
    let template = CreateElectionTemplate {
        login_state: ctx.login_state(),
    }
    .render()
    .map_err(|_| ApiError::InternalServerError)?;

    Ok(Html(template))
}
#[derive(Serialize)]
pub struct PostCreateElectionResponse {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct PostVotePayload {
    name: String,
}

#[debug_handler]
async fn post_create_election(
    state: State<AppState>,
    cookies: Cookies,
    context: Ctx,
    Form(post_vote_payload): Form<PostVotePayload>,
) -> ApiResult<Redirect> {
    // Ensure that user is not logged in
    match context.login_state() {
        LoginState::Voter { .. } => Err(ApiError::AuthFailed(AuthFailedError::TokenNotExpected)),
        LoginState::Admin { .. } => Err(ApiError::AuthFailed(AuthFailedError::TokenNotExpected)),
        LoginState::NotLoggedIn => Ok(()),
    }?;

    let mut tx = state.db.begin().await?;

    let res = sqlx::query_as!(
        PostCreateElectionResponse,
        "
        INSERT INTO election (name) VALUES ($1)
        RETURNING *;  
        ",
        post_vote_payload.name
    )
    .fetch_one(&mut *tx)
    .await?;

    let current_timestamp = Utc::now();
    let expiration_time = current_timestamp + chrono::Duration::hours(TOKEN_EXPIRY_DURATION_HOURS);

    let claims = JsonWebTokenClaims {
        exp: expiration_time.timestamp(),
        iat: current_timestamp.timestamp(),
        election_id: res.id.into(),
    };

    let token_result = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.hmac_key.as_bytes()),
    );

    let _ = token_result
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
        .map_err(|_| ApiError::InternalServerError)?;

    let _txresult = tx.commit().await?;

    Ok(Redirect::to("/"))
}

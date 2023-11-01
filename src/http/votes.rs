use std::ops::DerefMut;

use crate::api_types::ApiError;
use crate::api_types::AuthFailedError::MissingToken;
use crate::models::LoginState;
use crate::{
    api_types::{
        ApiError::AlreadyVoted, ApiError::AuthFailed, ApiError::InternalServerError, ApiResult,
    },
    ctx::Ctx,
    http::AppState,
    middleware::require_is_voter::require_is_voter,
};
use axum::{debug_handler, extract::State, middleware::from_fn, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::error::ErrorKind;
use sqlx::QueryBuilder;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/votes/", post(post_vote))
        .route_layer(from_fn(require_is_voter))
}
#[derive(Serialize)]

struct PostVoteResponse {
    candidates: Vec<String>,
    voting_id: i32,
}

#[derive(Deserialize)]

struct PostVotePayload {
    candidates: Vec<String>,
    voting_id: i32,
}

#[debug_handler]
async fn post_vote(
    state: State<AppState>,
    context: Ctx,
    Json(post_vote_payload): Json<PostVotePayload>,
) -> ApiResult<Json<PostVoteResponse>> {
    let token = match context.login_state() {
        LoginState::Voter { token, .. } => Ok(token),
        _ => Err(ApiError::TokenNotFound),
    }?;

    // Start a transaction to add tuples to both vote and has_voted
    let mut tx = state.db.begin().await?;

    // If the voter does not vote for anyone ( candidates = [] ), then don't insert anything into vote, and the tx wont fail to syntax error
    let _insert_vote = if post_vote_payload.candidates.len() > 0 {
        QueryBuilder::new("INSERT INTO vote(candidate_name, voting_id, rank) ")
            .push_values(
                post_vote_payload.candidates.iter().enumerate(),
                |mut query_builder, (index, candidate_name)| {
                    query_builder
                        .push_bind(candidate_name)
                        .push_bind(post_vote_payload.voting_id.clone())
                        .push_bind(index as i32 + 1); // ranks start at 1 (rank int DEFAULT 1 defined in the db schema), not 0
                },
            )
            .build()
            // Executor impl for Transaction has been removed since 0.7. Add a dereference to the inner connection which still impl's Transaction
            // https://github.com/launchbadge/sqlx/issues/2672
            // https://github.com/launchbadge/sqlx/blob/main/CHANGELOG.md#breaking
            .execute(tx.deref_mut())
            .await?;
    };

    // Duplicate key error prevents us from voting twice, and the tx fails
    let _insert_has_voted = sqlx::query!(
        "INSERT INTO has_voted (token_token, voting_id) VALUES ($1, $2) ",
        token,
        post_vote_payload.voting_id
    )
    .execute(tx.deref_mut())
    .await
    .map_err(|e| match e {
        // Handle unique key error (trying to vote twice)
        sqlx::Error::Database(err) if err.kind() == ErrorKind::UniqueViolation => AlreadyVoted,
        _ => InternalServerError,
    })?;

    tx.commit().await?;

    // TODO add meaningful error messages
    Ok(Json(PostVoteResponse {
        candidates: post_vote_payload.candidates,
        voting_id: post_vote_payload.voting_id,
    }))
}

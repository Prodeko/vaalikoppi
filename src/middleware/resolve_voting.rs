use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};

use crate::{
    api_types::{
        ApiError::{self, VotingNotFound},
        ApiResult,
    },
    http::AppState,
    models::{CandidateId, ElectionId, Voting, VotingId, VotingStateWithoutResults},
};

pub async fn resolve_voting<B>(
    election_id: ElectionId,
    Path(id): Path<VotingId>,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    let voting = sqlx::query_as!(
        Voting,
        "
        SELECT
            v.id,
            v.election_id,
            v.name,
            v.description,
            v.state AS \"state: VotingStateWithoutResults\",
            v.created_at,
            v.hide_vote_counts,
            v.number_of_winners,
            COALESCE(NULLIF(ARRAY_AGG(c.name), '{NULL}'), '{}') AS \"candidates!: Vec<CandidateId>\"
        FROM voting as v LEFT JOIN candidate as c
            ON v.id = c.voting_id
        WHERE v.id = $1 and v.election_id = $2
        GROUP BY v.id;
        ",
        id,
        election_id as ElectionId
    )
    .fetch_optional(&state.db)
    .await?;

    voting.map(|v| {
        req.extensions_mut().insert(v);
    });

    Ok(next.run(req).await)
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Voting {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> ApiResult<Self> {
        parts
            .extensions
            .get::<Voting>()
            .map(|voting| voting.clone())
            .ok_or(VotingNotFound)
    }
}

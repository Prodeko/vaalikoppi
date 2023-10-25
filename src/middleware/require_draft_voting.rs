use axum::{
    extract::{Path, State},
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::{
    error::{
        Error::{VotingAlreadyOpened, VotingNotFound},
        Result,
    },
    http::AppState,
    models::{CandidateId, SqlxVotingState, Voting, VotingId, VotingState::Draft},
};

pub async fn require_draft_voting<B>(
    Path(id): Path<VotingId>,
    state: State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    let voting_query_result = sqlx::query_as!(
        Voting,
        "
        SELECT
            v.id,
            v.name,
            v.description,
            v.state AS \"state: SqlxVotingState\",
            v.created_at,
            v.hide_vote_counts,
            COALESCE(NULLIF(ARRAY_AGG(c.name), '{NULL}'), '{}') AS \"candidates!: Vec<CandidateId>\"
        FROM voting as v LEFT JOIN candidate as c
            ON v.id = c.voting_id
        WHERE v.id = $1
        GROUP BY v.id;
        ",
        id
    )
    .fetch_optional(&state.db)
    .await?;

    let voting = match voting_query_result {
        Some(voting) => match voting.state {
            Draft => Ok(voting),
            _ => Err(VotingAlreadyOpened),
        },
        None => Err(VotingNotFound),
    }?;

    req.extensions_mut().insert(voting);

    Ok(next.run(req).await)
}

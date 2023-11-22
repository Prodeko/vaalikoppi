use askama::Template;
use axum::{extract::State, response::Html, routing::get, Router};

use crate::{
    api_types::{ApiError, ApiResult},
    ctx::Ctx,
    models::{CandidateId, LoginState},
};

use super::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new().route("/", get(get_audit))
}

async fn get_audit(ctx: Ctx, state: State<AppState>) -> ApiResult<Html<String>> {
    let votes = sqlx::query_as!(
        AuditRow,
        "
        SELECT
            voting.name as voting_name,
            vote.id,
            COALESCE(NULLIF(ARRAY_AGG(vote.candidate_name ORDER BY vote.rank ASC), '{null}'), '{}') as \"vote!: Vec<CandidateId>\",
            hide_vote_counts
        FROM
            voting INNER JOIN vote ON voting.id = vote.voting_id
        GROUP BY voting.name, vote.id, voting.hide_vote_counts
        "
    )
    .fetch_all(&state.db)
    .await?;

    AuditTemplate {
        login_state: ctx.login_state(),
        votes,
    }
    .render()
    .map(|h| Html(h))
    .map_err(|e| ApiError::TemplatingError(e))
}

struct AuditRow {
    pub voting_name: String,
    pub id: String,
    pub vote: Vec<CandidateId>,
    pub hide_vote_counts: bool,
}

#[derive(Template)]
#[template(path = "pages/audit.html")]
struct AuditTemplate {
    login_state: LoginState,
    votes: Vec<AuditRow>,
}

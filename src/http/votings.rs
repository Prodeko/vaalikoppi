use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Path, State},
    middleware::from_fn,
    response::Html,
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

use crate::{
    error::{Error, Result},
    middleware::require_admin_token::require_admin,
    models::{
        CandidateId, CandidateResultData, PassingCandidateResult, SqlxVotingState, Voting,
        VotingId, VotingResult, VotingRoundResult, VotingState,
    },
};

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", post(post_voting))
        .route("/:id", put(put_voting))
        .route("/:id", delete(delete_voting))
        .route_layer(from_fn(require_admin))
        .route("/", get(get_votings))
}

async fn post_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn put_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn delete_voting(state: State<AppState>, id: Path<u64>) -> Html<String> {
    Html("hello world".to_string())
}
async fn get_votings(state: State<AppState>) -> askama::Html {
    get_all_votings_html(state.db.clone()).await
}

#[derive(Template)]
#[template(path = "voting-list.html", ext = "html")]
struct VotingsTemplate<'a> {
    open_votings: Vec<Voting>,
    closed_votings: Vec<Voting>,
    ended_votings: Vec<VotingResult>,
    csrf_token: &'a str,
    is_admin: bool,
}

async fn get_all_votings_html(db: Pool<Postgres>) -> Result<Html<String>> {
    let rows = sqlx::query!(
        "
        with passing_candidate_result_data AS (
            SELECT p.*, c.vote_count
            FROM passing_candidate_result as p INNER JOIN candidate_result_data as c
                ON p.voting_id = c.voting_id
                AND p.name = c.name
                AND p.round = c.round
        ),
        dropped_candidates AS (
            SELECT name, round, voting_id, vote_count
            FROM candidate_result_data
            WHERE (voting_id, round, name) NOT IN (
                SELECT voting_id, round, name
                FROM passing_candidate_result
            )
        ),
        round_results AS (

            --- It seems that SQLx is not able to infer that these are nullable
            --- I suspect that this is related to MATCH SIMPLE in foreign keys.
            --- Thus we'll force nullability as per https://docs.rs/sqlx/latest/sqlx/macro.query.html#force-nullable
            --- Similarly we'll force not-null on the COALESCEs.    
            SELECT
                r.voting_id as voting_id,
                r.round as round,
                d.name as dropped_candidate_name,
                d.vote_count as dropped_candidate_vote_count,
                COALESCE(ARRAY_AGG(p.name), '{}') as candidate_names,
                COALESCE(ARRAY_AGG(p.is_selected), '{}') as candidate_is_selected,
                COALESCE(ARRAY_AGG(p.vote_count), '{}') as candidate_vote_count
            FROM
                voting_round_result as r
                LEFT JOIN passing_candidate_result_data as p
                    ON r.voting_id = p.voting_id AND r.round = p.round
                LEFT JOIN dropped_candidates as d
                    ON r.voting_id = d.voting_id AND r.round = d.round
            GROUP BY (r.voting_id, r.round, d.name, d.vote_count)
        ),
        candidates_by_voting AS (
            SELECT v.id, COALESCE(ARRAY_AGG(c.name), '{}') as candidates
            FROM voting AS v LEFT JOIN candidate AS c ON v.id = c.voting_id
            GROUP BY v.id
        )

        SELECT
            v.id as \"id!: VotingId\",
            v.state as \"state!: SqlxVotingState\",
            v.name as \"name!: String\", 
            v.description as \"description!: String\",
            v.created_at as \"created_at!: DateTime<Utc>\",
            v.hide_vote_counts as \"hide_vote_counts!: bool\",
            v.number_of_votes as \"number_of_votes!: i32\",
            c.candidates as \"candidates!: Vec<CandidateId>\",
            r.round as \"round?: i32\",
            r.dropped_candidate_name as \"dropped_candidate_name?: String\",
            r.dropped_candidate_vote_count as \"dropped_candidate_vote_count?: f64\",
            r.candidate_names as \"candidate_names?: Vec<CandidateId>\",
            r.candidate_is_selected as \"candidate_is_selected?: Vec<bool>\",
            r.candidate_vote_count as \"candidate_vote_count?: Vec<f64>\"
        FROM
            voting AS v
            LEFT JOIN candidates_by_voting AS c ON v.id = c.id
            LEFT JOIN round_results AS r ON v.id = r.voting_id
        ORDER BY round ASC;
        "
        ).fetch_all(&db);

    let mut votings: HashMap<VotingId, Voting> = HashMap::new();

    let rows = rows.await?;
    rows.into_iter().try_for_each(|rec| {
        let candidate_results = rec
            .candidate_names
            .zip(rec.candidate_is_selected)
            .zip(rec.candidate_vote_count)
            .map(|((names, is_selecteds), vote_counts)| {
                names
                    .into_iter()
                    .zip(is_selecteds.into_iter())
                    .zip(vote_counts.into_iter())
                    .map(|((name, is_selected), vote_count)| PassingCandidateResult {
                        data: CandidateResultData { name, vote_count },
                        is_selected,
                    })
                    .collect::<Vec<PassingCandidateResult>>()
            });

        let dropped_candidate: Option<CandidateResultData> = rec
            .dropped_candidate_name
            .zip(rec.dropped_candidate_vote_count)
            .map(|(name, vote_count)| CandidateResultData { name, vote_count });

        let round_result: Option<VotingRoundResult> =
            rec.round
                .zip(candidate_results)
                .map(|(round, results)| VotingRoundResult {
                    round,
                    dropped_candidate,
                    candidate_results: results,
                });

        let voting = votings.get_mut(&rec.id);

        // TODO clean up these ugly matches
        match voting {
            Some(v) => match (&mut v.state, round_result) {
                (VotingState::Draft, None) => Ok(()),
                (VotingState::Draft, Some(_)) => Err(Error::CorruptDatabaseError),
                (VotingState::Open, None) => Ok(()),
                (VotingState::Open, Some(_)) => Err(Error::CorruptDatabaseError),
                (
                    VotingState::Closed {
                        round_results: _,
                        winners: _,
                    },
                    None,
                ) => Err(Error::CorruptDatabaseError),
                (
                    VotingState::Closed {
                        round_results,
                        winners: _,
                    },
                    Some(result),
                ) => {
                    round_results.push(result);
                    Ok(())
                }
            },

            None => {
                let state = match rec.state {
                    SqlxVotingState::Draft => VotingState::Draft,
                    SqlxVotingState::Open => VotingState::Open,
                    SqlxVotingState::Closed => VotingState::Closed {
                        round_results: Vec::new(),
                        winners: Vec::new(),
                    },
                };

                let voting = Voting {
                    id: rec.id,
                    candidates: rec.candidates,
                    name: rec.name,
                    description: rec.description,
                    state,
                    created_at: rec.created_at,
                    hide_vote_counts: rec.hide_vote_counts,
                    number_of_votes: rec.number_of_votes,
                };

                votings.insert(rec.id, voting);
                Ok(())
            }
        }
    })?;

    // .map(|row| {
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //
    //     VotingRoundResult {
    //         candidate_results,
    //         dropped_candidate,
    //         round: row.round,
    //     }
    // }).fetch_all(&db);

    //let (open_votings, closed_votings): (Vec<_>, Vec<_>) =
    //    votings.into_iter().partition(|v| v.is_open);
    //
    //let template = VotingsTemplate {
    //    closed_votings,
    //    open_votings,
    //    ended_votings:
    //}
    //template
    //    .render()
    //    .map(|s| Html(s))
    //    .map_err(|_| Error::InternalServerError)
    todo!()
}

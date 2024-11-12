use ::serde::{Deserialize, Serialize};
use axum::{
    debug_handler,
    extract::{Path, State},
    middleware::{from_fn, from_fn_with_state},
    response::Html,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::Utc;
use sqlx::{postgres::PgRow, Executor, Pool, Postgres, QueryBuilder, Row};
use std::collections::HashMap;
use validator::Validate;

use askama::Template;
use chrono::DateTime;

use crate::{
    api_types::{
        ApiError::{self, InternalServerError},
        ApiResult,
    },
    ctx::Ctx,
    helpers::calculate_stv_result,
    middleware::{require_is_admin::require_is_admin, resolve_voting::resolve_voting},
    models::{
        Alias, CandidateId, CandidateResultData, LoginState, PassingCandidateResult, Voting,
        VotingCreate, VotingForVoterTemplate, VotingId, VotingResult, VotingRoundResult,
        VotingState, VotingStateWithoutResults, VotingUpdate,
    },
};

use super::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:id", patch(patch_voting))
        .route("/:id", delete(delete_voting))
        .route_layer(from_fn_with_state(state, resolve_voting))
        .route("/", post(post_voting))
        .route_layer(from_fn(require_is_admin))
        .route("/", get(get_votings))
}

#[debug_handler]
async fn post_voting(
    state: State<AppState>,
    Json(voting_create): Json<VotingCreate>,
) -> ApiResult<Json<Voting>> {
    let voting_state = voting_create
        .state
        .unwrap_or(VotingStateWithoutResults::Draft);

    match voting_state {
        VotingStateWithoutResults::Draft => Ok(()),
        VotingStateWithoutResults::Open => {
            if voting_create
                .candidates
                .as_ref()
                .map(|v| v.is_empty())
                .unwrap_or(true)
            {
                Err(ApiError::InvalidInput)
            } else {
                Ok(())
            }
        }
        VotingStateWithoutResults::Closed => Err(ApiError::InvalidInput),
    }?;

    let mut tx = state.db.begin().await?;

    let mut voting = sqlx::query!(
        "
        INSERT INTO voting (name, description, state, created_at, hide_vote_counts, number_of_winners)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            name,
            description,
            state AS \"state: VotingStateWithoutResults\",
            created_at,
            hide_vote_counts,
            number_of_winners;
        ",
        voting_create.name,
        voting_create.description,
        voting_state as VotingStateWithoutResults,
        Utc::now(),
        voting_create.hide_vote_counts,
        voting_create.number_of_winners,
    )
    .map(|row| Voting {
        id: row.id,
        name: row.name,
        description: row.description,
        state: VotingState::from(row.state),
        created_at: row.created_at,
        hide_vote_counts: row.hide_vote_counts,
        number_of_winners: row.number_of_winners,
        candidates: vec![],
    })
    .fetch_one(&mut *tx)
    .await?;

    let candidates = insert_candidates_into_db(
        voting.id,
        voting_create.candidates.unwrap_or(vec![]),
        &mut *tx,
    )
    .await?;

    voting.candidates = candidates;

    tx.commit().await?;

    Ok(Json(voting))
}

async fn insert_candidates_into_db<T>(
    voting_id: VotingId,
    candidates: Vec<CandidateId>,
    executor: &mut T,
) -> ApiResult<Vec<CandidateId>>
where
    for<'e> &'e mut T: Executor<'e, Database = Postgres>,
{
    if candidates.is_empty() {
        return Ok(vec![]);
    }

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO candidate(voting_id, name) ");

    query_builder.push_values(candidates, |mut b, name| {
        b.push_bind(voting_id).push_bind(name);
    });

    query_builder.push("RETURNING name");

    query_builder
        .build()
        .map(|row: PgRow| row.get::<CandidateId, &'static str>("name"))
        .fetch_all(executor)
        .await
        .map_err(|e| e.into())
}

#[debug_handler]
async fn patch_voting(
    existing_voting: Voting,
    state: State<AppState>,
    _id: Path<VotingId>,
    Json(voting_update): Json<VotingUpdate>,
) -> ApiResult<Json<Voting>> {
    let res = existing_voting
        .handle_patch(state.db.clone(), voting_update)
        .await
        .map(|v| Json(v))?;
    Ok(res)
}

#[debug_handler]
pub async fn get_votings(ctx: Ctx, state: State<AppState>) -> ApiResult<Html<String>> {
    match ctx.login_state() {
        LoginState::NotLoggedIn => todo!(),
        LoginState::Voter { .. } => {
            get_votings_list_template(state.db.clone(), ctx.login_state(), None)
                .await?
                .render()
                .map(|html| Html(html))
                .map_err(|_| ApiError::InternalServerError)
        }
        LoginState::Admin => get_admin_votings_list_template(state.db.clone(), ctx.login_state())
            .await?
            .render()
            .map(|html| Html(html))
            .map_err(|_| ApiError::InternalServerError),
    }
}

struct VotingStateResult {
    state: VotingStateWithoutResults,
}

impl Voting {
    pub async fn handle_patch(
        &self,
        db: Pool<Postgres>,
        voting_update: VotingUpdate,
    ) -> ApiResult<Voting> {
        println!("handle patch for voting {}", self.id);
        match (
            &self.state,
            voting_update
                .state
                .unwrap_or_else(|| self.state.clone().into()),
        ) {
            (VotingState::Closed { .. }, _) => Err(ApiError::VotingAlreadyClosed),
            (_, VotingStateWithoutResults::Closed) => {
                self.try_close_voting(db, voting_update).await
            }
            (_, _) => self.try_modify_and_reset_votes(db, voting_update).await,
        }
    }

    async fn try_close_voting(
        &self,
        db: Pool<Postgres>,
        voting_update: VotingUpdate,
    ) -> ApiResult<Voting> {
        if self.state != VotingStateWithoutResults::Open {
            return Err(ApiError::InvalidInput);
        }
        if voting_update.state != Some(VotingStateWithoutResults::Closed) {
            return Err(ApiError::InternalServerError);
        }

        let mut clone = self.clone();
        let voting_update_without_state_change = VotingUpdate {
            state: None,
            ..voting_update.clone()
        };

        let no_other_fields_modified = clone == voting_update_without_state_change;

        if !no_other_fields_modified {
            return Err(ApiError::InvalidInput);
        }

        let mut tx = db.begin().await?;

        let count_of_active_tokens_that_have_not_voted = sqlx::query!(
            "
            WITH activated_tokens_without_vote AS (
                SELECT token
                FROM token
                WHERE
                    state = 'activated'::token_state
                    AND token NOT IN (
                        SELECT token_token AS token
                        FROM has_voted
                        WHERE voting_id = $1
                    )
            )
            SELECT count(*) as \"count!\"
            FROM activated_tokens_without_vote
            ",
            self.id
        )
        .map(|row| row.count)
        .fetch_one(&mut *tx)
        .await?;

        if count_of_active_tokens_that_have_not_voted != 0 {
            return Err(ApiError::NotAllActiveTokensHaveVoted);
        }

        let votes = sqlx::query!(
            "
            SELECT COALESCE(NULLIF(ARRAY_AGG(candidate_name ORDER BY rank), '{NULL}'), '{}') AS \"vote!: Vec<CandidateId>\"
            FROM vote
            WHERE voting_id = $1
            GROUP BY id
            ",
            self.id,
        ).map(|row| {
            row.vote
        }).fetch_all(&mut *tx).await?;

        let number_of_winners: usize = self
            .number_of_winners
            .try_into()
            .map_err(|_| ApiError::InternalServerError)?;

        let round_results =
            calculate_stv_result(self.candidates.clone(), votes, number_of_winners)?.round_results;

        let mut winning_candidates = vec![];
        let mut passing_candidates = vec![];
        let mut dropped_candidates = vec![];

        let round_results_references = round_results.iter().collect::<Vec<_>>();
        round_results.iter().for_each(|r| {
            r.candidate_results.iter().for_each(|c| {
                if c.is_selected {
                    winning_candidates.push((&c.data, r.round));
                } else {
                    passing_candidates.push((&c.data, r.round));
                }
            });

            r.dropped_candidate.iter().for_each(|c| {
                dropped_candidates.push((c, r.round));
            });
        });

        QueryBuilder::new("INSERT INTO voting_round_result (voting_id, round)")
            .push_values(round_results_references.clone(), |mut b, res| {
                b.push_bind(self.id).push_bind(res.round);
            })
            .build()
            .execute(&mut *tx)
            .await?;

        let all_candidate_data = [
            winning_candidates.clone(),
            passing_candidates.clone(),
            dropped_candidates,
        ]
        .concat();

        QueryBuilder::new(
            "INSERT INTO candidate_result_data (name, round, voting_id, vote_count, is_draw)",
        )
        .push_values(all_candidate_data, |mut b, (result, round)| {
            b.push_bind(&result.name)
                .push_bind(round)
                .push_bind(self.id)
                .push_bind(result.vote_count)
                .push_bind(result.is_draw);
        })
        .build()
        .execute(&mut *tx)
        .await?;

        if winning_candidates.len() > 0 {
            QueryBuilder::new(
                "INSERT INTO passing_candidate_result (name, round, voting_id, is_selected)",
            )
            .push_values(winning_candidates, |mut b, (result, round)| {
                b.push_bind(&result.name)
                    .push_bind(round)
                    .push_bind(self.id)
                    .push_bind(true);
            })
            .build()
            .execute(&mut *tx)
            .await?;
        }

        if passing_candidates.len() > 0 {
            QueryBuilder::new(
                "INSERT INTO passing_candidate_result (name, round, voting_id, is_selected)",
            )
            .push_values(passing_candidates, |mut b, (result, round)| {
                b.push_bind(&result.name)
                    .push_bind(round)
                    .push_bind(self.id)
                    .push_bind(false);
            })
            .build()
            .execute(&mut *tx)
            .await?;
        }

        let updated_voting = sqlx::query_as!(
            VotingStateResult,
            "
                UPDATE voting
                SET state = 'closed'::voting_state
                WHERE id = $1
                returning state AS \"state: VotingStateWithoutResults\";
                ",
            self.id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        clone.state = VotingState::from(updated_voting.state);
        return Ok(clone);
    }

    async fn try_modify_and_reset_votes(
        &self,
        db: Pool<Postgres>,
        voting_update: VotingUpdate,
    ) -> ApiResult<Voting> {
        let voting_state = voting_update.state.unwrap_or(self.state.clone().into());
        match voting_state {
            VotingStateWithoutResults::Open => {
                if voting_update
                    .candidates
                    .as_ref()
                    .unwrap_or(&self.candidates)
                    .is_empty()
                {
                    Err(ApiError::InvalidInput)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }?;

        let mut tx = db.begin().await?;

        sqlx::query!("DELETE FROM candidate WHERE voting_id = $1", self.id)
            .execute(&mut *tx)
            .await?;

        sqlx::query!("DELETE FROM has_voted WHERE voting_id = $1", self.id)
            .execute(&mut *tx)
            .await?;

        let candidates = insert_candidates_into_db(
            self.id,
            voting_update.candidates.unwrap_or(self.candidates.clone()),
            &mut *tx,
        )
        .await?;

        let voting = sqlx::query!(
            "
            UPDATE voting
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                state = COALESCE($4, state),
                hide_vote_counts = COALESCE($5, hide_vote_counts),
                number_of_winners = COALESCE($6, number_of_winners)
            WHERE id = $1
            RETURNING
                id,
                name,
                description,
                state AS \"state: VotingStateWithoutResults\",
                created_at,
                hide_vote_counts,
                number_of_winners;
            ",
            self.id,
            voting_update.name,
            voting_update.description,
            voting_update.state as Option<VotingStateWithoutResults>,
            voting_update.hide_vote_counts,
            voting_update.number_of_winners,
        )
        .map(|row| Voting {
            id: row.id,
            name: row.name,
            description: row.description,
            state: VotingState::from(row.state),
            created_at: row.created_at,
            hide_vote_counts: row.hide_vote_counts,
            number_of_winners: row.number_of_winners,
            candidates: candidates.clone(),
        })
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(voting)
    }
}

struct DeletedRowsCount {
    count: i64,
}

#[debug_handler]
async fn delete_voting(
    existing_voting: Voting,
    state: State<AppState>,
    id: Path<VotingId>,
) -> ApiResult<()> {
    match existing_voting.state {
        VotingState::Draft => Ok(()),
        VotingState::Open => Ok(()),
        _ => Err(ApiError::VotingAlreadyClosed),
    }?;

    let query_result = sqlx::query_as!(
        DeletedRowsCount,
        "
        WITH deleted_rows AS (
            DELETE FROM voting
            WHERE voting.id = $1
            RETURNING *
        )
        SELECT COUNT(*) AS \"count!\"
        FROM deleted_rows;
        ",
        id.0
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| ApiError::VotingNotFound)?;

    match query_result.count {
        ..=-1 => Err(ApiError::InternalServerError),
        0 => Err(ApiError::VotingNotFound),
        1.. => Ok(()),
    }
}

#[derive(Template)]
#[template(path = "components/voting-list.html")]

pub struct VotingListTemplate {
    pub open_votings: Vec<VotingForVoterTemplate>,
    pub draft_votings: Vec<Voting>,
    pub closed_votings: Vec<Voting>,
    pub login_state: LoginState,
    pub newly_created_vote_uuids: Option<Vec<String>>,
}

struct VotingData {
    open_votings: Vec<VotingForVoterTemplate>,
    closed_votings: Vec<Voting>,
    draft_votings: Vec<Voting>,
}
async fn get_voting_data(
    db: Pool<Postgres>,
    login_state: &LoginState,
) -> Result<VotingData, ApiError> {
    let token = match &login_state {
        LoginState::Voter { token, .. } => token.clone(),
        _ => "".to_string(), // TODO bad practices, this is used to ignore token when using this function for admin voting template
    };
    let rows = sqlx::query!(
        "
        with passing_candidate_result_data AS (
            SELECT p.*, c.vote_count, c.is_draw
            FROM passing_candidate_result as p INNER JOIN candidate_result_data as c
                ON p.voting_id = c.voting_id
                AND p.name = c.name
                AND p.round = c.round
        ),
        dropped_candidates AS (
            SELECT name, round, voting_id, vote_count, is_draw
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
                d.is_draw as dropped_candidate_is_draw,
                COALESCE(NULLIF(ARRAY_AGG(p.name), '{NULL}'), '{}') as candidate_names,
                COALESCE(NULLIF(ARRAY_AGG(p.is_selected), '{NULL}'), '{}') as candidate_is_selected,
                COALESCE(NULLIF(ARRAY_AGG(p.vote_count), '{NULL}'), '{}') as candidate_vote_count,
                COALESCE(NULLIF(ARRAY_AGG(p.is_draw), '{NULL}'), '{}') as candidate_is_draw
            FROM
                voting_round_result as r
                LEFT JOIN passing_candidate_result_data as p
                    ON r.voting_id = p.voting_id AND r.round = p.round
                LEFT JOIN dropped_candidates as d
                    ON r.voting_id = d.voting_id AND r.round = d.round
            GROUP BY (r.voting_id, r.round, d.name, d.vote_count, d.is_draw)
        ),
        voting_with_candidates AS (
            SELECT v.*, COALESCE(NULLIF(ARRAY_AGG(c.name), '{NULL}'), '{}') as candidates
            FROM voting AS v LEFT JOIN candidate AS c ON v.id = c.voting_id
            GROUP BY v.id
        )

        --- The return type of ARRAY_AGG has to be mangled so it returns an empty list. This is not exactly type safe.
        SELECT
            v.id as \"id!: VotingId\",
            v.state as \"state!: VotingStateWithoutResults\",
            v.name as \"name!: String\", 
            v.description as \"description!: String\",
            v.created_at as \"created_at!: DateTime<Utc>\",
            v.hide_vote_counts as \"hide_vote_counts!: bool\",
            v.number_of_winners,
            v.candidates as \"candidates!: Vec<CandidateId>\",
            r.round as \"round?: i32\",
            r.dropped_candidate_name as \"dropped_candidate_name?: String\",
            r.dropped_candidate_vote_count as \"dropped_candidate_vote_count?: f64\",
            r.dropped_candidate_is_draw as \"dropped_candidate_is_draw?: bool\",
            r.candidate_names as \"candidate_names?: Vec<CandidateId>\",
            r.candidate_is_selected as \"candidate_is_selected?: Vec<bool>\",
            r.candidate_vote_count as \"candidate_vote_count?: Vec<f64>\",
            r.candidate_is_draw as \"candidate_is_draw?: Vec<bool>\",
            (hv.token_token = $1) as \"you_have_voted?: bool\"
        FROM
            voting_with_candidates AS v            
            LEFT JOIN round_results AS r ON v.id = r.voting_id
            LEFT JOIN has_voted hv on v.id = hv.voting_id and hv.token_token = $1
        ORDER BY round ASC, candidate_vote_count DESC, v.created_at ASC;
        ", token
        ).fetch_all(&db);

    let mut votings: HashMap<VotingId, VotingForVoterTemplate> = HashMap::new();

    let rows = rows.await?;
    //println!("{:#?}", rows);
    rows.into_iter().try_for_each(|rec| {
        // println!("rec: {:#?}", rec);
        let candidate_results = rec
            .candidate_names
            .zip(rec.candidate_is_selected)
            .zip(rec.candidate_vote_count)
            .zip(rec.candidate_is_draw)
            .map(|(((names, is_selecteds), vote_counts), is_draws)| {
                names
                    .into_iter()
                    .zip(is_selecteds.into_iter())
                    .zip(vote_counts.into_iter())
                    .zip(is_draws.into_iter())
                    .map(
                        |(((name, is_selected), vote_count), is_draw)| PassingCandidateResult {
                            data: CandidateResultData {
                                name,
                                vote_count,
                                is_draw,
                            },
                            is_selected,
                        },
                    )
                    .collect::<Vec<PassingCandidateResult>>()
            });
        // println!("candidate_results: {:#?}", candidate_results);
        let dropped_candidate: Option<CandidateResultData> = rec
            .dropped_candidate_name
            .zip(rec.dropped_candidate_vote_count)
            .zip(rec.dropped_candidate_is_draw)
            .map(|((name, vote_count), is_draw)| CandidateResultData {
                name,
                vote_count,
                is_draw,
            });
        // println!("dropped_candidate: {:#?}", dropped_candidate);
        let round_result: Option<VotingRoundResult> =
            rec.round
                .zip(candidate_results)
                .map(|(round, results)| VotingRoundResult {
                    round,
                    dropped_candidate,
                    candidate_results: results,
                });
        // println!("round_result: {:#?}", round_result);

        let voting = votings.get_mut(&rec.id);

        match voting {
            Some(v) => match (&mut v.state, round_result) {
                (VotingState::Draft, None) => Ok(()),
                (VotingState::Draft, Some(_)) => Err(ApiError::CorruptDatabaseError),
                (VotingState::Open, None) => Ok(()),
                (VotingState::Open, Some(_)) => Err(ApiError::CorruptDatabaseError),
                (VotingState::Closed(_), None) => Err(ApiError::CorruptDatabaseError),
                (VotingState::Closed(existing_result), Some(result)) => {
                    existing_result.winners.extend(
                        result
                            .candidate_results
                            .iter()
                            .filter(|c| c.is_selected)
                            .map(|c| c.data.name.to_owned()),
                    );
                    existing_result.round_results.push(result);
                    Ok(())
                }
            },
            None => {
                let state = match (rec.state, round_result) {
                    (VotingStateWithoutResults::Closed, Some(round_result)) => {
                        Ok(VotingState::Closed(VotingResult {
                            winners: round_result
                                .candidate_results
                                .iter()
                                .filter(|c| c.is_selected)
                                .map(|c| c.data.name.to_owned())
                                .collect(),
                            round_results: vec![round_result],
                        }))
                    }
                    (VotingStateWithoutResults::Open, None) => Ok(VotingState::Open),
                    (VotingStateWithoutResults::Draft, None) => Ok(VotingState::Draft),
                    _ => Err(ApiError::CorruptDatabaseError),
                }?;

                let voting = VotingForVoterTemplate {
                    id: rec.id,
                    candidates: rec.candidates,
                    name: rec.name,
                    description: rec.description,
                    state,
                    created_at: rec.created_at,
                    hide_vote_counts: rec.hide_vote_counts,
                    you_have_voted: rec.you_have_voted.unwrap_or(false),
                    number_of_winners: rec.number_of_winners,
                };

                votings.insert(rec.id, voting);
                Ok(())
            }
        }
    })?;

    let mut draft_votings: Vec<Voting> = vec![];
    let mut open_votings: Vec<VotingForVoterTemplate> = vec![];
    let mut results_votings: Vec<Voting> = vec![];

    votings.values().for_each(|f| match &f.state {
        VotingState::Draft => draft_votings.push(f.to_owned().into()),
        VotingState::Open => open_votings.push(f.to_owned()),
        VotingState::Closed(VotingResult { .. }) => results_votings.push(f.to_owned().into()),
    });

    Ok(VotingData {
        open_votings,
        draft_votings,
        closed_votings: results_votings,
    })
}

pub async fn get_votings_list_template(
    db: Pool<Postgres>,
    login_state: LoginState,
    newly_created_vote_uuids: Option<Vec<String>>,
) -> ApiResult<VotingListTemplate> {
    let data = get_voting_data(db, &login_state).await?;

    let template = VotingListTemplate {
        open_votings: data.open_votings,
        draft_votings: data.draft_votings,
        closed_votings: data.closed_votings,
        // csrf_token: todo!(),
        login_state: login_state,
        newly_created_vote_uuids,
    };

    // println!("{:#?}", template.closed_votings);
    Ok(template)
}

// ---------------------------------------

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminOpenVoting {
    pub id: VotingId,           // voting
    pub name: String,           // voting
    pub description: String,    // voting
    pub state: VotingState,     // voting
    pub hide_vote_counts: bool, // voting
    pub number_of_winners: i32, // voting

    pub total_votes: i32,                         // has_voted
    pub eligible_token_count: i32,                // live count of activated tokens
    pub candidates: Vec<CandidateId>,             // candidate
    pub tokens_not_voted: Vec<AdminDisplayToken>, // token (active) join has_voted
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminDraftVoting {
    pub id: VotingId,
    pub name: String,
    pub description: String,
    pub state: VotingState,
    pub candidates: Vec<CandidateId>,
    pub hide_vote_counts: bool,
    pub number_of_winners: i32,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct AdminDisplayToken {
    pub token: String, // token
    pub alias: String, // token
}

#[derive(Template)]
#[template(path = "components/admin-voting-list.html")]
pub struct AdminVotingListTemplate {
    pub draft_votings: Vec<AdminDraftVoting>,
    pub open_votings: Vec<AdminOpenVoting>,
    pub closed_votings: Vec<Voting>, // ??
    pub login_state: LoginState,
}

pub async fn get_admin_votings_list_template(
    db: Pool<Postgres>,
    login_state: LoginState,
) -> ApiResult<AdminVotingListTemplate> {
    let rows = sqlx::query!(
        "
        with u_t as ( -- unused tokens for each voting
            select 
                v.id, 
                coalesce(nullif(
                    -- filter all unactivated and voided tokens here
                    array_agg(row(t.token, t.alias)) filter (where t.state = 'activated'::token_state), 
                '{NULL}'), '{}') as unused_tokens
            from voting v 
            cross join token t
            left join has_voted hv on hv.token_token = t.token and hv.voting_id = v.id
            where hv.token_token is null
            group by v.id
        ), 
        t_v as ( -- count of votes for each voting
            select
                v.id,
                count(hv.token_token) as total_votes
            from voting v
            left join has_voted hv on v.id = hv.voting_id
            group by v.id
        ),
        v_c as ( -- votings and their corresponding candidates
            select
                v.id,
                v.name,
                v.description,
                v.state,
                v.hide_vote_counts,
                v.number_of_winners,
                coalesce(nullif(array_agg(c.name), '{null}'), '{}') as candidates
            from voting v
            left join candidate c on v.id = c.voting_id
            group by v.id
        )
        select 
            v_c.id,
            v_c.name,
            v_c.description,
            v_c.state as \"voting_state!: VotingStateWithoutResults\",
            v_c.candidates as \"candidates!: Vec<String>\",
            v_c.hide_vote_counts,
            v_c.number_of_winners,
            COALESCE(u_t.unused_tokens, '{}') as \"unused_tokens!: Vec<(String, Alias)>\",
            t_v.total_votes
        from v_c natural join t_v left join u_t
            on v_c.id = u_t.id;
        "
    )
    .fetch_all(&db)
    .await?;

    let count_of_live_tokens = sqlx::query!(
        "
        select
            count(*)
        from token 
        WHERE state = 'activated'::token_state;
        ",
    )
    .fetch_one(&db)
    .await?
    .count
    .ok_or(InternalServerError)? as i32;

    let data = get_voting_data(db, &login_state).await?;

    let mut open_votings: Vec<AdminOpenVoting> = vec![];
    let mut draft_votings: Vec<AdminDraftVoting> = vec![];
    let closed_votings: Vec<Voting> = data.closed_votings;

    rows.iter().for_each(|row| match row.voting_state {
        VotingStateWithoutResults::Closed => (),
        VotingStateWithoutResults::Draft => draft_votings.push(AdminDraftVoting {
            id: row.id,
            name: row.name.clone(),
            description: row.description.clone(),
            state: row.voting_state.into(),
            hide_vote_counts: row.hide_vote_counts,
            number_of_winners: row.number_of_winners,
            candidates: row.candidates.clone(),
        }),
        VotingStateWithoutResults::Open => open_votings.push(AdminOpenVoting {
            id: row.id,
            description: row.description.clone(),
            name: row.name.clone(),
            state: row.voting_state.into(),
            hide_vote_counts: row.hide_vote_counts,
            number_of_winners: row.number_of_winners,
            total_votes: row.total_votes.map_or(-1 as i32, |t| t as i32),
            eligible_token_count: count_of_live_tokens,
            candidates: row.candidates.clone(),
            tokens_not_voted: row
                .unused_tokens
                .iter()
                .map(|t| AdminDisplayToken {
                    token: t.0.clone(),
                    alias: t.1.clone().unwrap_or("".to_string()),
                })
                .collect(),
        }),
    });

    Ok(AdminVotingListTemplate {
        open_votings,
        draft_votings,
        closed_votings,
        login_state,
    })
}

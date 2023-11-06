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
        ApiError::{self, AuthFailed, InternalServerError},
        ApiResult,
        AuthFailedError::InvalidToken,
    },
    ctx::Ctx,
    middleware::{require_is_admin::require_is_admin, resolve_voting::resolve_voting},
    models::{
        Alias, CandidateId, CandidateResultData, LoginState, PassingCandidateResult, Voting,
        VotingCreate, VotingForVoterTemplate, VotingId, VotingRoundResult, VotingState,
        VotingStateWithoutResults, VotingUpdate,
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
        INSERT INTO voting (name, description, state, created_at, hide_vote_counts)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            name,
            description,
            state AS \"state: VotingStateWithoutResults\",
            created_at,
            hide_vote_counts;
        ",
        voting_create.name,
        voting_create.description,
        voting_state as VotingStateWithoutResults,
        Utc::now(),
        voting_create.hide_vote_counts,
    )
    .map(|row| Voting {
        id: row.id,
        name: row.name,
        description: row.description,
        state: VotingState::from(row.state),
        created_at: row.created_at,
        hide_vote_counts: row.hide_vote_counts,
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
            (
                VotingState::Closed {
                    round_results: _,
                    winners: _,
                },
                _,
            ) => Err(ApiError::VotingAlreadyClosed),
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

        if no_other_fields_modified {
            let result = sqlx::query!(
                "
                UPDATE voting
                SET state = 'closed'::voting_state
                WHERE id = $1
                returning state AS \"state: VotingStateWithoutResults\";
                ",
                self.id
            )
            .fetch_one(&db)
            .await?;

            clone.state = VotingState::from(result.state);
            return Ok(clone);
        } else {
            return Err(ApiError::InvalidInput);
        }
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
                hide_vote_counts = COALESCE($5, hide_vote_counts)
            WHERE id = $1
            RETURNING
                id,
                name,
                description,
                state AS \"state: VotingStateWithoutResults\",
                created_at,
                hide_vote_counts;
            ",
            self.id,
            voting_update.name,
            voting_update.description,
            voting_update.state as Option<VotingStateWithoutResults>,
            voting_update.hide_vote_counts,
        )
        .map(|row| Voting {
            id: row.id,
            name: row.name,
            description: row.description,
            state: VotingState::from(row.state),
            created_at: row.created_at,
            hide_vote_counts: row.hide_vote_counts,
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
            WHERE voting.id = $1 AND voting.state = 'draft'::voting_state
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

#[derive(Debug)]
pub struct VotingResult {
    pub voting: Voting,
    pub round_results: Vec<VotingRoundResult>,
    pub winners: Vec<CandidateId>,
}

#[derive(Template)]
#[template(path = "components/voting-list.html")]

pub struct VotingListTemplate {
    pub open_votings: Vec<VotingForVoterTemplate>,
    pub draft_votings: Vec<Voting>,
    pub closed_votings: Vec<VotingResult>,
    pub login_state: LoginState,
    pub newly_created_vote_uuids: Option<Vec<String>>,
}

pub async fn get_votings_list_template(
    db: Pool<Postgres>,
    login_state: LoginState,
    newly_created_vote_uuids: Option<Vec<String>>,
) -> ApiResult<VotingListTemplate> {
    let token = match &login_state {
        LoginState::Voter { token, .. } => token.clone(),
        _ => Err(AuthFailed(InvalidToken))?,
    };
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
                COALESCE(NULLIF(ARRAY_AGG(p.name), '{NULL}'), '{}') as candidate_names,
                COALESCE(NULLIF(ARRAY_AGG(p.is_selected), '{NULL}'), '{}') as candidate_is_selected,
                COALESCE(NULLIF(ARRAY_AGG(p.vote_count), '{NULL}'), '{}') as candidate_vote_count
            FROM
                voting_round_result as r
                LEFT JOIN passing_candidate_result_data as p
                    ON r.voting_id = p.voting_id AND r.round = p.round
                LEFT JOIN dropped_candidates as d
                    ON r.voting_id = d.voting_id AND r.round = d.round
            GROUP BY (r.voting_id, r.round, d.name, d.vote_count)
        ),
        candidates_by_voting AS (
            SELECT v.id, COALESCE(NULLIF(ARRAY_AGG(c.name), '{NULL}'), '{}') as candidates
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
            c.candidates as \"candidates!: Vec<CandidateId>\",
            r.round as \"round?: i32\",
            r.dropped_candidate_name as \"dropped_candidate_name?: String\",
            r.dropped_candidate_vote_count as \"dropped_candidate_vote_count?: f64\",
            r.candidate_names as \"candidate_names?: Vec<CandidateId>\",
            r.candidate_is_selected as \"candidate_is_selected?: Vec<bool>\",
            r.candidate_vote_count as \"candidate_vote_count?: Vec<f64>\",
            (hv.token_token = $1) as \"you_have_voted?: bool\"
        FROM
            voting AS v
            INNER JOIN candidates_by_voting AS c ON v.id = c.id
            LEFT JOIN round_results AS r ON v.id = r.voting_id
            LEFT JOIN has_voted hv on v.id = hv.voting_id and hv.token_token = $1
        ORDER BY round, v.created_at ASC;
        ", token
        ).fetch_all(&db);

    let mut votings: HashMap<VotingId, VotingForVoterTemplate> = HashMap::new();

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

        match voting {
            Some(v) => match (&mut v.state, round_result) {
                (VotingState::Draft, None) => Ok(()),
                (VotingState::Draft, Some(_)) => Err(ApiError::CorruptDatabaseError),
                (VotingState::Open, None) => Ok(()),
                (VotingState::Open, Some(_)) => Err(ApiError::CorruptDatabaseError),
                (
                    VotingState::Closed {
                        round_results: _,
                        winners: _,
                    },
                    None,
                ) => Err(ApiError::CorruptDatabaseError),
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
                    VotingStateWithoutResults::Draft => VotingState::Draft,
                    VotingStateWithoutResults::Open => VotingState::Open,
                    VotingStateWithoutResults::Closed => VotingState::Closed {
                        round_results: Vec::new(),
                        winners: Vec::new(),
                    },
                };

                let voting = VotingForVoterTemplate {
                    id: rec.id,
                    candidates: rec.candidates,
                    name: rec.name,
                    description: rec.description,
                    state,
                    created_at: rec.created_at,
                    hide_vote_counts: rec.hide_vote_counts,
                    you_have_voted: rec.you_have_voted.unwrap_or(false),
                };

                votings.insert(rec.id, voting);
                Ok(())
            }
        }
    })?;

    let mut draft_votings: Vec<Voting> = vec![];
    let mut open_votings: Vec<VotingForVoterTemplate> = vec![];
    let mut results_votings: Vec<VotingResult> = vec![];

    votings.values().for_each(|f| match &f.state {
        VotingState::Draft => draft_votings.push(f.to_owned().into()),
        VotingState::Open => open_votings.push(f.clone()),
        VotingState::Closed {
            round_results,
            winners,
        } => results_votings.push(VotingResult {
            round_results: round_results.to_owned(),
            winners: winners.to_owned(),
            voting: f.to_owned().into(),
        }),
    });

    let template = VotingListTemplate {
        open_votings,
        draft_votings,
        closed_votings: results_votings,
        // csrf_token: todo!(),
        login_state: login_state,
        newly_created_vote_uuids,
    };

    Ok(template)
}

// ---------------------------------------

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminOpenVoting {
    pub id: VotingId,        // voting
    pub name: String,        // voting
    pub description: String, // voting
    pub state: VotingState,  // voting

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
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
pub struct AdminDisplayToken {
    pub token: String, // token
    pub alias: String, // token
}

#[derive(Template)]
#[template(path = "components/admin-voting-list.html")]
pub struct AdminVotingListTemplate {
    pub open_votings: Vec<AdminOpenVoting>,
    pub draft_votings: Vec<AdminDraftVoting>,
    pub closed_votings: Vec<VotingResult>, // ??
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
            u_t.unused_tokens as \"unused_tokens!: Vec<(String, Alias)>\",
            t_v.total_votes
        from v_c natural join u_t natural join t_v;
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

    let mut open_votings: Vec<AdminOpenVoting> = vec![];
    let mut draft_votings: Vec<AdminDraftVoting> = vec![];
    let mut closed_votings: Vec<VotingResult> = vec![];

    rows.iter().for_each(|row| match row.voting_state {
        VotingStateWithoutResults::Closed => (),
        VotingStateWithoutResults::Draft => draft_votings.push(AdminDraftVoting {
            id: row.id,
            name: row.name.clone(),
            description: row.description.clone(),
            state: row.voting_state.into(),
            candidates: row.candidates.clone(),
        }),
        VotingStateWithoutResults::Open => open_votings.push(AdminOpenVoting {
            id: row.id,
            description: row.description.clone(),
            name: row.name.clone(),
            state: row.voting_state.into(),
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

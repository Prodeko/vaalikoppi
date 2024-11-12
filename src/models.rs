use chrono::{DateTime, Utc};
use float_cmp::approx_eq;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;
use uuid::Uuid;
use validator::Validate;

pub type CandidateId = String;
pub type VotingId = i32;
pub type TokenId = i32;
pub type Alias = Option<String>;

static CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvxyz";
static TOKEN_LENGTH: usize = 9;

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let get_one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(get_one_char)
        .take(TOKEN_LENGTH)
        .collect::<String>()
}

trait Voteable {
    fn calculate_results(&self) -> Vec<CandidateId>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LoginState {
    NotLoggedIn,
    // We do not store the whole Token struct because it can represent states that are invalid.
    // TODO it might be better to create a new struct, e.g., "ValidToken",
    // That only contains the data that we want to represent a valid voter login state.
    Voter { token: String, alias: String },
    Admin,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "voting_state", rename_all = "lowercase")]
pub enum VotingStateWithoutResults {
    Draft,
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "token_state", rename_all = "lowercase")]
pub enum TokenState {
    Unactivated,
    Activated,
    Voided,
}

impl From<VotingState> for VotingStateWithoutResults {
    fn from(value: VotingState) -> Self {
        match value {
            VotingState::Draft => VotingStateWithoutResults::Draft,
            VotingState::Open => VotingStateWithoutResults::Open,
            VotingState::Closed { .. } => VotingStateWithoutResults::Closed,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VotingState {
    Draft,
    Open,
    #[serde(rename_all = "camelCase")]
    Closed(VotingResult),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct VotingResult {
    pub round_results: Vec<VotingRoundResult>,
    pub winners: Vec<CandidateId>,
}

impl From<VotingStateWithoutResults> for VotingState {
    fn from(value: VotingStateWithoutResults) -> Self {
        match value {
            VotingStateWithoutResults::Draft => Self::Draft,
            VotingStateWithoutResults::Open => Self::Open,
            VotingStateWithoutResults::Closed => Self::Closed(VotingResult {
                round_results: vec![],
                winners: vec![],
            }),
        }
    }
}

impl PartialEq<VotingStateWithoutResults> for VotingState {
    fn eq(&self, other: &VotingStateWithoutResults) -> bool {
        match (self, other) {
            (VotingState::Draft, VotingStateWithoutResults::Draft) => true,
            (VotingState::Open, VotingStateWithoutResults::Open) => true,
            (VotingState::Closed { .. }, VotingStateWithoutResults::Closed) => true,
            _ => false,
        }
    }
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Voting {
    pub id: VotingId,
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    #[validate(length(min = 0, max = 128))]
    pub description: String,
    pub state: VotingState,
    pub created_at: DateTime<Utc>,
    pub hide_vote_counts: bool,
    pub number_of_winners: i32,
    pub candidates: Vec<CandidateId>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VotingForVoterTemplate {
    pub id: VotingId,
    pub name: String,
    pub description: String,
    pub state: VotingState,
    pub created_at: DateTime<Utc>,
    pub hide_vote_counts: bool,
    pub candidates: Vec<CandidateId>,
    pub number_of_winners: i32,
    pub you_have_voted: bool,
}

impl From<VotingForVoterTemplate> for Voting {
    fn from(value: VotingForVoterTemplate) -> Self {
        Voting {
            id: value.id,
            name: value.name,
            description: value.description,
            state: value.state,
            created_at: value.created_at,
            hide_vote_counts: value.hide_vote_counts,
            candidates: value.candidates,
            number_of_winners: value.number_of_winners,
        }
    }
}

// TODO this screams for a macro
impl PartialEq<VotingUpdate> for Voting {
    fn eq(&self, other: &VotingUpdate) -> bool {
        let other_clone = other.clone();
        other_clone.name.map(|n| self.name == n).unwrap_or(true)
            && other_clone
                .description
                .map(|d| self.description == d)
                .unwrap_or(true)
            && other_clone.state.map(|s| self.state == s).unwrap_or(true)
            && other_clone
                .hide_vote_counts
                .map(|h| self.hide_vote_counts == h)
                .unwrap_or(true)
            && other_clone
                .number_of_winners
                .map(|h| self.number_of_winners == h)
                .unwrap_or(true)
            && other_clone
                .candidates
                .map(|c| self.candidates == c)
                .unwrap_or(true)
    }
}

#[derive(Validate, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VotingCreate {
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    #[validate(length(min = 0, max = 128))]
    pub description: String,
    pub state: Option<VotingStateWithoutResults>,
    pub hide_vote_counts: bool,
    pub number_of_winners: i32,
    pub candidates: Option<Vec<CandidateId>>,
}

#[derive(Validate, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VotingUpdate {
    #[validate(length(min = 1, max = 128))]
    pub name: Option<String>,
    #[validate(length(min = 0, max = 128))]
    pub description: Option<String>,
    pub state: Option<VotingStateWithoutResults>,
    pub hide_vote_counts: Option<bool>,
    pub number_of_winners: Option<i32>,
    pub candidates: Option<Vec<CandidateId>>,
}

#[derive(Debug)]
pub struct HasVoted {
    pub voting_id: VotingId,
    pub token_id: TokenId,
    pub has_voted: bool,
}

#[derive(Debug)]
pub struct Vote {
    pub voting_id: VotingId,
    pub id: Uuid,
    pub candidate: CandidateId,
    pub rank: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub id: TokenId,
    pub token: String,
    pub state: TokenState,
    pub alias: Alias,
}

#[derive(Debug, Deserialize)]
pub struct TokenUpdate {
    pub state: TokenState,
}

#[derive(Debug)]
pub struct VoteCastStatus {
    pub voting: VotingId,
    pub token: TokenId,
    pub has_voted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateResultData {
    pub name: CandidateId,
    pub vote_count: f64,
    pub is_draw: bool,
}

impl PartialEq for CandidateResultData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && approx_eq!(f64, self.vote_count, other.vote_count, epsilon = 0.000001)
    }
}

impl Eq for CandidateResultData {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PassingCandidateResult {
    pub data: CandidateResultData,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VotingRoundResult {
    pub round: i32,
    pub candidate_results: Vec<PassingCandidateResult>,
    pub dropped_candidate: Option<CandidateResultData>,
}

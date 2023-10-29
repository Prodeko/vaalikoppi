use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter;
use validator::Validate;

pub type CandidateId = String;
pub type VotingId = i32;
pub type TokenId = String;
pub type Alias = Option<String>;

static CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvxyz";
static TOKEN_LENGTH: usize = 6;

pub fn generate_token() -> TokenId {
    let mut rng = rand::thread_rng();
    let get_one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(get_one_char)
        .take(TOKEN_LENGTH)
        .collect::<String>()
}

trait Voteable {
    fn calculate_results(&self) -> Vec<CandidateId>;
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "voting_state", rename_all = "lowercase")]
pub enum VotingStateWithoutResults {
    Draft,
    Open,
    Closed,
}

impl From<VotingState> for VotingStateWithoutResults {
    fn from(value: VotingState) -> Self {
        match value {
            VotingState::Draft => VotingStateWithoutResults::Draft,
            VotingState::Open => VotingStateWithoutResults::Open,
            VotingState::Closed {
                round_results: _,
                winners: _,
            } => VotingStateWithoutResults::Closed,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VotingState {
    Draft,
    Open,
    #[serde(rename_all = "camelCase")]
    Closed {
        round_results: Vec<VotingRoundResult>,
        winners: Vec<CandidateId>,
    },
}

impl From<VotingStateWithoutResults> for VotingState {
    fn from(value: VotingStateWithoutResults) -> Self {
        match value {
            VotingStateWithoutResults::Draft => Self::Draft,
            VotingStateWithoutResults::Open => Self::Open,
            VotingStateWithoutResults::Closed => Self::Closed {
                round_results: vec![],
                winners: vec![],
            },
        }
    }
}

impl PartialEq<VotingStateWithoutResults> for VotingState {
    fn eq(&self, other: &VotingStateWithoutResults) -> bool {
        match (self, other) {
            (VotingState::Draft, VotingStateWithoutResults::Draft) => true,
            (VotingState::Open, VotingStateWithoutResults::Open) => true,
            (
                VotingState::Closed {
                    round_results: _,
                    winners: _,
                },
                VotingStateWithoutResults::Closed,
            ) => true,
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
    pub candidates: Vec<CandidateId>,
}

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
    pub candidate: CandidateId,
    pub rank: i32,
}

#[derive(Debug)]
pub struct Token {
    pub id: TokenId,
    pub is_activated: bool,
    pub is_trashed: bool,
    pub alias: Alias,
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassingCandidateResult {
    pub data: CandidateResultData,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VotingRoundResult {
    pub round: i32,
    pub candidate_results: Vec<PassingCandidateResult>,
    pub dropped_candidate: Option<CandidateResultData>,
}

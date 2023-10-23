use chrono::{DateTime, Utc};
use rand::Rng;
use std::iter;
use validator::Validate;

pub type CandidateId = String;
pub type VotingId = i32;
pub type TokenId = String;

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

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "voting_state", rename_all = "lowercase")]
pub enum SqlxVotingState {
    Draft,
    Open,
    Closed,
}

#[derive(Debug)]
pub enum VotingState {
    Draft,
    Open,
    Closed {
        round_results: Vec<VotingRoundResult>,
        winners: Vec<CandidateId>,
    },
}

#[derive(Validate, Debug)]
pub struct Voting {
    pub id: VotingId,
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    #[validate(length(min = 0, max = 128))]
    pub description: String,
    pub state: VotingState,
    pub created_at: DateTime<Utc>,
    pub hide_vote_counts: bool,
    pub number_of_votes: i32,
    pub candidates: Vec<CandidateId>,
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
}

#[derive(Debug)]
pub struct VoteCastStatus {
    pub voting: VotingId,
    pub token: TokenId,
    pub has_voted: bool,
}

#[derive(Debug)]
pub struct CandidateResultData {
    pub name: CandidateId,
    pub vote_count: f64,
}

#[derive(Debug)]
pub struct PassingCandidateResult {
    pub data: CandidateResultData,
    pub is_selected: bool,
}

#[derive(Debug)]
pub struct VotingRoundResult {
    pub round: i32,
    pub candidate_results: Vec<PassingCandidateResult>,
    pub dropped_candidate: Option<CandidateResultData>,
}

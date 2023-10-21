use chrono::{DateTime, Utc};
use validator::Validate;

type CandidateId = String;
type VotingId = String;
type TokenId = String;

trait Voteable {
    fn calculate_results(&self) -> Vec<CandidateId>;
}

#[derive(Validate, Debug)]
pub struct Voting {
    pub id: VotingId,
    #[validate(length(min = 1, max = 128))]
    pub name: String,
    #[validate(length(min = 0, max = 128))]
    pub description: String,
    pub is_open: bool,
    pub created_at: DateTime<Utc>,
    pub hide_vote_counts: bool,
    pub number_of_votes: u8,
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
    pub rank: u8,
}

#[derive(Debug)]
pub struct Token {
    pub id: TokenId,
    pub is_activated: bool,
    pub is_trashed: bool,
}

#[derive(Debug)]
pub struct VotingStatus {
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
    pub is_elected: bool,
}

#[derive(Debug)]
pub struct VotingRoundResult {
    pub round: u64,
    pub candidate_results: Vec<PassingCandidateResult>,
    pub dropped_candidate: Option<CandidateResultData>,
}

#[derive(Debug)]
pub struct VotingResult<'a> {
    pub voting: &'a Voting,
    pub round_results: Vec<VotingRoundResult>,
    pub winners: Vec<CandidateId>,
}

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
    id: VotingId,
    #[validate(length(min = 1, max = 128))]
    name: String,
    #[validate(length(min = 0, max = 128))]
    description: String,
    is_open: bool,
    created_at: DateTime<Utc>,
    hide_vote_counts: bool,
    number_of_votes: u8,
    candidates: Vec<CandidateId>,
}

#[derive(Debug)]
pub struct HasVoted {
    voting_id: VotingId,
    token_id: TokenId,
    has_voted: bool,
}

#[derive(Debug)]
pub struct Vote {
    voting_id: VotingId,
    candidate: CandidateId,
    rank: u8,
}

#[derive(Debug)]
pub struct Token {
    id: TokenId,
    is_activated: bool,
    is_trashed: bool,
}

#[derive(Debug)]
pub struct VotingStatus {
    voting: VotingId,
    token: TokenId,
    has_voted: bool,
}

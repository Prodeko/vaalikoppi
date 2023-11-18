DROP TABLE IF EXISTS passing_candidate_result;

ALTER TABLE
    voting_round_result DROP CONSTRAINT fk_ensure_dropped_candidate_has_data;

DROP TABLE IF EXISTS candidate_result_data;

DROP TABLE IF EXISTS voting_round_result;

DROP TABLE IF EXISTS has_voted;

DROP TABLE IF EXISTS vote;

DROP TABLE IF EXISTS candidate;

DROP INDEX IF EXISTS token_token_hash_index;

DROP TABLE IF EXISTS token;

DROP TABLE IF EXISTS voting;

DROP TYPE IF EXISTS token_state;

DROP TYPE IF EXISTS voting_state;
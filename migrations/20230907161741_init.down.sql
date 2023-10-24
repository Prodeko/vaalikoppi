DROP TABLE passing_candidate_result;

ALTER TABLE
    voting_round_result DROP CONSTRAINT fk_ensure_dropped_candidate_has_data;

DROP TABLE candidate_result_data;

DROP TABLE voting_round_result;

DROP TABLE has_voted;

DROP TABLE vote;

DROP TABLE candidate;

DROP TABLE token;

DROP TABLE voting;

DROP TYPE voting_state;
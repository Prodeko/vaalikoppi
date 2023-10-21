-- Add up migration script here
CREATE TABLE voting_round_result (
    voting_id integer NOT NULL REFERENCES voting,
    round integer NOT NULL,
    dropped_candidate_name text,
    PRIMARY KEY (voting_id, round)
);

CREATE TABLE candidate_result_data (
    name text NOT NULL,
    round integer NOT NULL,
    voting_id integer NOT NULL,
    vote_count double precision NOT NULL,
    FOREIGN KEY (voting_id, round) REFERENCES voting_round_result(voting_id, round),
    FOREIGN KEY (voting_id, name) REFERENCES candidate(voting_id, name),
    PRIMARY KEY (name, round, voting_id)
);

ALTER TABLE
    voting_round_result
ADD
    CONSTRAINT fk_ensure_dropped_candidate_has_data FOREIGN KEY (voting_id, dropped_candidate_name, round) REFERENCES candidate_result_data(voting_id, name, round);

CREATE TABLE passing_candidate_result (
    voting_id integer NOT NULL,
    name text NOT NULL,
    round integer NOT NULL,
    is_selected BOOLEAN NOT NULL,
    FOREIGN KEY (voting_id, name, round) REFERENCES candidate_result_data(voting_id, name, round),
    PRIMARY KEY (voting_id, name, round)
);
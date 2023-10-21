-- Add up migration script here
CREATE TABLE voting_round_result (
    voting_id int REFERENCES voting NOT NULL,
    round int REFERENCES voting_result NOT NULL,
    dropped_candidate_name text,
    FOREIGN KEY dropped_candidate_name REFERENCES candidate_result_data(name),
    PRIMARY KEY (voting_id, round)
);

CREATE TABLE candidate_result_data (
    name text NOT NULL PRIMARY KEY,
    round int NOT NULL,
    voting_id int NOT NULL,
    vote_count double precision NOT NULL,
    FOREIGN KEY (round, voting_id) REFERENCES voting_round_result,
    PRIMARY KEY (name, round, voting_id)
);

CREATE TABLE passing_candidate_result (
    voting_id int NOT NULL,
    name text NOT NULL,
    round int NOT NULL,
    FOREIGN KEY (voting_id, name, round) REFERENCES candidate_result_data,
    PRIMARY KEY (voting_id, name, round),
    is_selected BOOLEAN NOT NULL,
);
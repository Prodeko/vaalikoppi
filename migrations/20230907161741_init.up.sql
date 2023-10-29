-- Add migration script here
CREATE TYPE voting_state AS ENUM ('draft', 'open', 'closed');

CREATE TABLE voting (
    id SERIAL PRIMARY KEY NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    state voting_state NOT NULL,
    created_at timestamptz NOT NULL,
    hide_vote_counts boolean NOT NULL
);

CREATE TABLE token (
    id text PRIMARY KEY NOT NULL,
    is_activated boolean NOT NULL,
    is_trashed boolean NOT NULL,
    alias text
);

CREATE TABLE candidate (
    name text NOT NULL,
    voting_id int REFERENCES voting ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (voting_id, name)
);

CREATE TABLE vote (
    id SERIAL NOT NULL PRIMARY KEY,
    candidate_name text NOT NULL,
    voting_id int NOT NULL,
    rank int DEFAULT 1,
    FOREIGN KEY (candidate_name, voting_id) REFERENCES candidate(name, voting_id) ON DELETE CASCADE
);

CREATE TABLE has_voted (
    token_id text REFERENCES token NOT NULL,
    voting_id int REFERENCES voting ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (token_id, voting_id)
);

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
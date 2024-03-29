-- Add migration script here
CREATE TYPE voting_state AS ENUM ('draft', 'open', 'closed');

CREATE TYPE token_state AS ENUM ('unactivated', 'activated', 'voided');

CREATE TABLE voting (
    id SERIAL PRIMARY KEY NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    state voting_state NOT NULL,
    created_at timestamptz NOT NULL,
    hide_vote_counts boolean NOT NULL,
    number_of_winners int NOT NULL
);

CREATE TABLE token (
    id SERIAL PRIMARY KEY NOT NULL,
    token text UNIQUE NOT NULL,
    --- TODO change this to "secret"
    state token_state NOT NULL,
    alias text UNIQUE
);

CREATE INDEX token_token_hash_index ON token USING hash(token);

CREATE TABLE candidate (
    name text NOT NULL,
    voting_id int REFERENCES voting ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (voting_id, name)
);

CREATE TABLE vote (
    id uuid NOT NULL,
    candidate_name text NOT NULL,
    voting_id int NOT NULL,
    rank int DEFAULT 1,
    PRIMARY KEY (voting_id, id, rank),
    FOREIGN KEY (candidate_name, voting_id) REFERENCES candidate(name, voting_id) ON DELETE CASCADE
);

CREATE TABLE has_voted (
    token_token TEXT NOT NULL,
    voting_id int REFERENCES voting ON DELETE CASCADE NOT NULL,
    FOREIGN KEY (token_token) REFERENCES token(token),
    PRIMARY KEY (token_token, voting_id)
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
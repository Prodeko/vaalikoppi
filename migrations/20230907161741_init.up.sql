-- Add migration script here
CREATE TABLE voting (
    id SERIAL PRIMARY KEY NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    is_open boolean NOT NULL,
    created_at timestamp NOT NULL,
    hide_vote_counts boolean NOT NULL,
    number_of_votes int NOT NULL
);

CREATE TABLE token (
    id text PRIMARY KEY NOT NULL,
    is_activated boolean NOT NULL,
    is_trashed boolean NOT NULL
);

CREATE TABLE candidate (
    name text NOT NULL,
    voting_id int REFERENCES voting NOT NULL,
    is_elected boolean NOT NULL,
    PRIMARY KEY (voting_id, name)
);

CREATE TABLE vote (
    id SERIAL PRIMARY KEY NOT NULL,
    candidate_name text NOT NULL,
    voting_id int NOT NULL,
    rank int DEFAULT 1,
    FOREIGN KEY (candidate_name, voting_id) REFERENCES candidate(name, voting_id)
);

CREATE TABLE has_voted (
    token_id text REFERENCES token NOT NULL,
    voting_id int REFERENCES voting NOT NULL,
    has_voted boolean NOT NULL,
    PRIMARY KEY (token_id, voting_id)
);
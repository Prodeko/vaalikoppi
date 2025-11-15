-- Add up migration script here

BEGIN TRANSACTION;

	CREATE TABLE election (
		id SERIAL PRIMARY KEY NOT NULL,
		name text NOT NULL,
		created_at timestamptz NOT NULL
	);

	ALTER TABLE voting
		ADD election_id int NOT NULL,
		ADD CONSTRAINT fk_link_voting_to_election FOREIGN KEY (election_id) REFERENCES election(id);

	ALTER TABLE token
		ADD election_id int NOT NULL,
		ADD constraint fk_link_token_to_election FOREIGN KEY (election_id) REFERENCES election(id);

COMMIT;

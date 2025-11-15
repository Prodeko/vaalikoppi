-- Add down migration script here

BEGIN TRANSACTION;

	ALTER TABLE token
		DROP COLUMN IF EXISTS election_id,
		DROP CONSTRAINT IF EXISTS fk_link_token_to_election;

	ALTER TABLE voting
		DROP COLUMN IF EXISTS election_id,
		DROP CONSTRAINT IF EXISTS fk_link_voting_to_election;

	DROP TABLE IF EXISTS election;

COMMIT;

-- Add down migration script here

BEGIN TRANSACTION;

	ALTER TABLE candidate_result_data
	DROP COLUMN IF EXISTS is_draw;

COMMIT;

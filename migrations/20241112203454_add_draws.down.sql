-- Add down migration script here

ALTER TABLE candidate_result_data
DROP COLUMN IF EXISTS is_draw;
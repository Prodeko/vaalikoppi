-- Add up migration script here

ALTER TABLE candidate_result_data
ADD is_draw boolean;

UPDATE candidate_result_data
SET is_draw = false -- Defaulting to false for old votings
WHERE is_draw IS NULL;

ALTER TABLE candidate_result_data
ALTER COLUMN is_draw SET NOT NULL;
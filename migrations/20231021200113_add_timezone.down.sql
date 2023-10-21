-- Add down migration script here
ALTER TABLE
    voting
ALTER COLUMN
    created_at TYPE timestamp;
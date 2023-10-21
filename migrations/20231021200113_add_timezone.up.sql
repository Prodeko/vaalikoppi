-- Add up migration script here
ALTER TABLE
    voting
ALTER COLUMN
    created_at TYPE timestamptz;
-- Add up migration script here
CREATE TYPE voting_state AS ENUM ('draft', 'open', 'closed');

ALTER TABLE
    voting
ADD
    COLUMN state voting_state;

UPDATE
    voting
SET
    state = CASE
        WHEN is_open IS TRUE THEN 'open' :: voting_state
        ELSE 'closed' :: voting_state
    END;

ALTER TABLE
    voting
ALTER COLUMN
    state
SET
    NOT NULL;

ALTER TABLE
    voting DROP COLUMN is_open;
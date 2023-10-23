-- Add down migration script here
-- Add back the "is_open" column
ALTER TABLE
    voting
ADD
    COLUMN is_open BOOLEAN;

-- Update the "is_open" column based on the value of the "state" column
UPDATE
    voting
SET
    is_open = CASE
        WHEN state = 'open' THEN TRUE
        ELSE FALSE
    END;

ALTER TABLE
    voting
ALTER COLUMN
    is_open
SET
    NOT NULL;

-- Drop the "state" column
ALTER TABLE
    voting DROP COLUMN state;

-- Drop the "voting_state" enum type
DROP TYPE voting_state;
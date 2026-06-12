-- 006_reply_votes.up.sql

CREATE TABLE IF NOT EXISTS reply_votes (
    id BIGINT PRIMARY KEY NOT NULL DEFAULT snowflake_id(),
    user_id BIGINT NOT NULL,
    reply_id BIGINT NOT NULL REFERENCES replies(id) ON DELETE CASCADE,
    direction SMALLINT NOT NULL CHECK (direction IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (user_id, reply_id)
);

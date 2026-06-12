-- 003_post_votes_and_tags.up.sql

-- Add tags and view_count to posts table
ALTER TABLE posts ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}';
ALTER TABLE posts ADD COLUMN IF NOT EXISTS view_count BIGINT DEFAULT 0;

-- Post votes table (one vote per user per post, direction = +1 or -1)
CREATE TABLE IF NOT EXISTS post_votes (
    id BIGINT PRIMARY KEY NOT NULL DEFAULT snowflake_id(),
    user_id BIGINT NOT NULL,
    post_id BIGINT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    direction SMALLINT NOT NULL CHECK (direction IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (user_id, post_id)
);

-- Comment votes table
CREATE TABLE IF NOT EXISTS comment_votes (
    id BIGINT PRIMARY KEY NOT NULL DEFAULT snowflake_id(),
    user_id BIGINT NOT NULL,
    comment_id BIGINT NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    direction SMALLINT NOT NULL CHECK (direction IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (user_id, comment_id)
);

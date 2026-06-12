-- 003_post_votes_and_tags.down.sql
ALTER TABLE posts DROP COLUMN IF EXISTS tags;
ALTER TABLE posts DROP COLUMN IF EXISTS view_count;
DROP TABLE IF EXISTS post_votes CASCADE;
DROP TABLE IF EXISTS comment_votes CASCADE;

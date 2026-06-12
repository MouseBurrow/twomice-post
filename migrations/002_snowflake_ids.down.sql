-- 1. Drop stored procedures
DROP FUNCTION IF EXISTS random_b62_5();
DROP FUNCTION IF EXISTS create_topic(TEXT, TEXT);
DROP FUNCTION IF EXISTS get_topic(TEXT);
DROP FUNCTION IF EXISTS get_all_topics();
DROP FUNCTION IF EXISTS create_post(BIGINT, TEXT, TEXT, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS get_post(TEXT, TEXT);
DROP FUNCTION IF EXISTS get_all_post(TEXT);
DROP FUNCTION IF EXISTS create_comment(BIGINT, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS get_all_comments(TEXT, TEXT);
DROP FUNCTION IF EXISTS create_reply(BIGINT, TEXT, TEXT, TEXT, TEXT);
DROP FUNCTION IF EXISTS get_replies(TEXT, TEXT, TEXT);

-- 2. Drop tables
DROP TABLE IF EXISTS replies CASCADE;
DROP TABLE IF EXISTS comments CASCADE;
DROP TABLE IF EXISTS posts CASCADE;
DROP TABLE IF EXISTS topics CASCADE;

-- 3. Drop snowflake helpers
DROP FUNCTION IF EXISTS snowflake_id();
DROP SEQUENCE IF EXISTS global_snowflake_seq;

-- 4. Recreate original UUID schema
CREATE TABLE topics
(
    id          UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name        TEXT UNIQUE      NOT NULL,
    description TEXT             NOT NULL,
    created_at  TIMESTAMPTZ               DEFAULT NOW(),
    deleted     BOOL                      DEFAULT FALSE
);

CREATE TABLE posts
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    creator_id UUID             NOT NULL,
    topic_id   UUID             NOT NULL REFERENCES topics (id) ON DELETE CASCADE,
    title      TEXT             NOT NULL,
    slug       TEXT             NOT NULL,
    content    TEXT             NOT NULL,
    image_url  TEXT,
    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,
    UNIQUE (topic_id, slug)
);

CREATE TABLE comments
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    hash       VARCHAR(5)       NOT NULL,
    sender_id  UUID             NOT NULL,
    post_id    UUID             NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    content    TEXT             NOT NULL,
    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,
    UNIQUE (post_id, hash)
);

CREATE TABLE replies
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    hash       VARCHAR(5)       NOT NULL,
    sender_id  UUID             NOT NULL,
    post_id    UUID             NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    comment_id UUID             NOT NULL REFERENCES comments (id) ON DELETE CASCADE,
    reply_id   UUID REFERENCES replies (id) ON DELETE CASCADE,
    content    TEXT             NOT NULL,
    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,
    UNIQUE (post_id, hash)
);

-- 5. Recreate original stored procedures with UUID
CREATE OR REPLACE FUNCTION random_b62_5() RETURNS TEXT LANGUAGE sql AS $$
    SELECT string_agg(
        substring('0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
            FROM (floor(random() * 62)::int + 1) FOR 1),
        '')
    FROM generate_series(1, 5)
$$;

CREATE OR REPLACE FUNCTION create_topic(p_name TEXT, p_description TEXT) RETURNS VOID LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO topics (name, description) VALUES (p_name, p_description);
EXCEPTION WHEN unique_violation THEN
    RAISE EXCEPTION 'Topic name already exists' USING ERRCODE = '23505';
END;
$$;

CREATE OR REPLACE FUNCTION get_topic(p_name TEXT) RETURNS TABLE(name TEXT, description TEXT, created_at TIMESTAMPTZ, deleted BOOL) LANGUAGE plpgsql AS $$
BEGIN
    RETURN QUERY SELECT t.name, t.description, t.created_at, t.deleted FROM topics t WHERE t.name = p_name ORDER BY t.created_at;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
END;
$$;

CREATE OR REPLACE FUNCTION get_all_topics() RETURNS TABLE(name TEXT, description TEXT, created_at TIMESTAMPTZ, deleted BOOLEAN) LANGUAGE plpgsql AS $$
BEGIN
    RETURN QUERY SELECT t.name, t.description, t.created_at, t.deleted FROM topics t ORDER BY t.created_at;
END;
$$;

CREATE OR REPLACE FUNCTION create_post(p_creator_id UUID, p_topic_name TEXT, p_title TEXT, p_slug TEXT, p_content TEXT, p_image_url TEXT) RETURNS TEXT LANGUAGE plpgsql AS $$
DECLARE
    d_topic_id UUID;
    d_final_slug TEXT;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    LOOP
        d_final_slug := p_slug || '-' || random_b62_5();
        BEGIN
            INSERT INTO posts (creator_id, topic_id, title, slug, content, image_url) VALUES (p_creator_id, d_topic_id, p_title, d_final_slug, p_content, p_image_url);
            RETURN d_final_slug;
        EXCEPTION WHEN unique_violation THEN CONTINUE; END;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION get_post(p_topic_name TEXT, p_post_slug TEXT) RETURNS TABLE(title TEXT, slug TEXT, content TEXT, image_url TEXT, created_at TIMESTAMPTZ, deleted BOOLEAN) LANGUAGE plpgsql AS $$
DECLARE d_topic_id UUID;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    RETURN QUERY SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted FROM posts p WHERE p.topic_id = d_topic_id AND p.slug = p_post_slug;
    IF NOT FOUND THEN RAISE EXCEPTION 'post_not_found' USING ERRCODE = 'P0001'; END IF;
END;
$$;

CREATE OR REPLACE FUNCTION get_all_post(p_topic_name TEXT) RETURNS TABLE(title TEXT, slug TEXT, content TEXT, image_url TEXT, created_at TIMESTAMPTZ, deleted BOOLEAN) LANGUAGE plpgsql AS $$
DECLARE d_topic_id UUID;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    RETURN QUERY SELECT p.title, p.slug, p.content, p.image_url, p.created_at, p.deleted FROM posts p WHERE p.topic_id = d_topic_id ORDER BY p.created_at;
END;
$$;

CREATE OR REPLACE FUNCTION create_comment(p_sender_id UUID, p_topic_name TEXT, p_post_slug TEXT, p_content TEXT) RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE
    d_topic_id UUID;
    d_post_id UUID;
    d_final_hash TEXT;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    SELECT id INTO d_post_id FROM posts WHERE topic_id = d_topic_id AND slug = p_post_slug;
    IF NOT FOUND THEN RAISE EXCEPTION 'post_not_found' USING ERRCODE = 'P0001'; END IF;
    LOOP
        d_final_hash := random_b62_5();
        BEGIN
            INSERT INTO comments (hash, sender_id, post_id, content) VALUES (d_final_hash, p_sender_id, d_post_id, p_content);
            RETURN;
        EXCEPTION WHEN unique_violation THEN CONTINUE; END;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION get_all_comments(p_topic_name TEXT, p_post_slug TEXT) RETURNS TABLE(hash VARCHAR(5), content TEXT, created_at TIMESTAMPTZ, deleted BOOLEAN) LANGUAGE plpgsql AS $$
DECLARE
    d_topic_id UUID;
    d_post_id UUID;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    SELECT id INTO d_post_id FROM posts WHERE topic_id = d_topic_id AND slug = p_post_slug;
    IF NOT FOUND THEN RAISE EXCEPTION 'post_not_found' USING ERRCODE = 'P0001'; END IF;
    RETURN QUERY SELECT c.hash, c.content, c.created_at, c.deleted FROM comments c WHERE c.post_id = d_post_id ORDER BY c.created_at;
END;
$$;

CREATE OR REPLACE FUNCTION create_reply(p_sender_id UUID, p_comment_hash TEXT, p_post_slug TEXT, p_topic_name TEXT, p_content TEXT) RETURNS VOID LANGUAGE plpgsql AS $$
DECLARE
    d_comment_id UUID;
    d_topic_id UUID;
    d_post_id UUID;
    final_hash TEXT;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    SELECT id INTO d_post_id FROM posts WHERE topic_id = d_topic_id AND slug = p_post_slug;
    IF NOT FOUND THEN RAISE EXCEPTION 'post_not_found' USING ERRCODE = 'P0001'; END IF;
    SELECT id INTO d_comment_id FROM comments WHERE hash = p_comment_hash;
    IF NOT FOUND THEN RAISE EXCEPTION 'comment_not_found' USING ERRCODE = 'P0002'; END IF;
    LOOP
        final_hash := random_b62_5();
        BEGIN
            INSERT INTO replies (hash, sender_id, post_id, comment_id, content) VALUES (final_hash, p_sender_id, d_post_id, d_comment_id, p_content);
            RETURN;
        EXCEPTION WHEN unique_violation THEN CONTINUE; END;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION get_replies(p_topic_name TEXT, p_post_slug TEXT, p_comment_hash TEXT) RETURNS TABLE(hash VARCHAR(5), content TEXT, created_at TIMESTAMPTZ, deleted BOOLEAN) LANGUAGE plpgsql AS $$
DECLARE
    d_comment_id UUID;
    d_topic_id UUID;
    d_post_id UUID;
BEGIN
    SELECT id INTO d_topic_id FROM topics WHERE name = p_topic_name;
    IF NOT FOUND THEN RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000'; END IF;
    SELECT id INTO d_post_id FROM posts WHERE topic_id = d_topic_id AND slug = p_post_slug;
    IF NOT FOUND THEN RAISE EXCEPTION 'post_not_found' USING ERRCODE = 'P0001'; END IF;
    SELECT c.id INTO d_comment_id FROM comments as c WHERE c.hash = p_comment_hash;
    IF NOT FOUND THEN RAISE EXCEPTION 'comment_not_found' USING ERRCODE = 'P0002'; END IF;
    RETURN QUERY SELECT r.hash, r.content, r.created_at, r.deleted FROM replies r WHERE r.post_id = d_post_id AND r.comment_id = d_comment_id ORDER BY r.created_at;
END;
$$;

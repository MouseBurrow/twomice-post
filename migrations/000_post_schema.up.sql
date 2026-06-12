CREATE EXTENSION IF NOT EXISTS pgcrypto;

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

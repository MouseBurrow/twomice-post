-- Topics table
CREATE TABLE topics
(
    id          UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),

    name        TEXT UNIQUE      NOT NULL,
    description TEXT             NOT NULL,

    created_at  TIMESTAMPTZ               DEFAULT NOW(),
    deleted     BOOL                      DEFAULT FALSE
);

-- Posts table
CREATE TABLE posts
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),
    creator_id UUID             NOT NULL REFERENCES auth.accounts (id) ON DELETE CASCADE,
    topic_id   UUID             NOT NULL REFERENCES topics (id) ON DELETE CASCADE,

    title      TEXT             NOT NULL,
    slug       TEXT             NOT NULL,
    content    TEXT             NOT NULL,
    image_url  TEXT,

    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,

    UNIQUE (topic_id, slug)
);

-- Comments table
CREATE TABLE comments
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),
    hash       VARCHAR(5)       NOT NULL,
    sender_id  UUID             NOT NULL REFERENCES auth.accounts (id) ON DELETE CASCADE,

    post_id    UUID             NOT NULL REFERENCES posts (id) ON DELETE CASCADE,

    content    TEXT             NOT NULL,

    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,

    UNIQUE (post_id, hash)
);

-- Replies table
CREATE TABLE replies
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),
    hash       VARCHAR(5)       NOT NULL,
    sender_id  UUID             NOT NULL REFERENCES auth.accounts (id) ON DELETE CASCADE,

    post_id    UUID             NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    comment_id UUID             NOT NULL REFERENCES comments (id) ON DELETE CASCADE,
    reply_id   UUID REFERENCES replies (id) ON DELETE CASCADE,

    content    TEXT             NOT NULL,

    created_at TIMESTAMPTZ               DEFAULT NOW(),
    deleted    BOOL                      DEFAULT FALSE,

    UNIQUE (post_id, hash)
);
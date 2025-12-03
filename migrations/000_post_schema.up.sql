-- Users table
CREATE TABLE posts
(
    id         UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),
    userid     UUID             NOT NULL REFERENCES auth.accounts (id) ON DELETE CASCADE,
    content    TEXT             NOT NULL,
    image_url  TEXT,
    created_at TIMESTAMP                 DEFAULT NOW()
);

-- Sessions table
CREATE TABLE comments
(
    id       UUID PRIMARY KEY NOT NULL DEFAULT extensions.uuid_generate_v4(),
    post_id  UUID             NOT NULL REFERENCES posts (id) ON DELETE CASCADE,
    user_id  UUID             NOT NULL REFERENCES auth.accounts (id) ON DELETE CASCADE,
    reply_id UUID             NOT NULL REFERENCES comments (id) ON DELETE CASCADE,
    content  TEXT             NOT NULL,
    is_reply BOOLEAN          NOT NULL
);

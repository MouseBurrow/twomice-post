CREATE TABLE IF NOT EXISTS topic_tags (
    topic_id BIGINT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    tag_name TEXT NOT NULL,
    PRIMARY KEY (topic_id, tag_name)
);

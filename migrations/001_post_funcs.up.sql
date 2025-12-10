CREATE OR REPLACE FUNCTION create_topic(
    p_name TEXT,
    p_description TEXT
)
    RETURNS VOID
    LANGUAGE plpgsql
AS
$$
BEGIN
    INSERT INTO topics (name, description)
    VALUES (p_name, p_description);
EXCEPTION
    WHEN unique_violation THEN
        RAISE EXCEPTION 'Topic name already exists' USING ERRCODE = '23505';
END;
$$;

CREATE OR REPLACE FUNCTION get_topic(
    p_name TEXT
)
    RETURNS TEXT
    LANGUAGE plpgsql
AS
$$
DECLARE
    topic_desc TEXT;
BEGIN
    SELECT description INTO topic_desc FROM topics WHERE name = p_name;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'topic_not_found' USING ERRCODE = 'P0000';
    END IF;

    RETURN topic_desc;
END;
$$;

CREATE OR REPLACE FUNCTION create_post(
    p_creator_id UUID,
    p_topic_id UUID,
    p_title TEXT,
    p_slug TEXT,
    p_content TEXT,
    p_image_url TEXT
)
    RETURNS TEXT
    LANGUAGE plpgsql
AS
$$
DECLARE
    final_slug TEXT;
BEGIN
    LOOP
        final_slug := p_slug || '-' || extensions.random_b62_5();

        BEGIN
            INSERT INTO posts (creator_id, topic_id, title, slug, content, image_url)
            VALUES (p_creator_id, p_topic_id, p_title, final_slug, p_content, p_image_url);

            RETURN final_slug;
        EXCEPTION
            WHEN unique_violation THEN
                CONTINUE;
        END;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION create_comment(
    p_sender_id UUID,
    p_post_id UUID,
    p_content TEXT
)
    RETURNS TEXT
    LANGUAGE plpgsql
AS
$$
DECLARE
    final_hash TEXT;
BEGIN
    LOOP
        final_hash := extensions.random_b62_5();

        BEGIN
            INSERT INTO comments (hash, sender_id, post_id, content)
            VALUES (final_hash, p_sender_id, p_post_id, p_content);

            RETURN final_hash;
        EXCEPTION
            WHEN unique_violation THEN
                CONTINUE;
        END;
    END LOOP;
END;
$$;

CREATE OR REPLACE FUNCTION create_reply(
    p_sender_id UUID,
    p_post_id UUID,
    p_comment_id UUID,
    p_reply_id UUID,
    p_content TEXT
)
    RETURNS TEXT
    LANGUAGE plpgsql
AS
$$
DECLARE
    final_hash TEXT;
BEGIN
    LOOP
        final_hash := extensions.random_b62_5();

        BEGIN
            INSERT INTO replies (hash, sender_id, post_id, comment_id, reply_id, content)
            VALUES (final_hash, p_sender_id, p_post_id, p_comment_id, p_reply_id, p_content);

            RETURN final_hash;

        EXCEPTION
            WHEN unique_violation THEN
                CONTINUE;
        END;
    END LOOP;
END;
$$;



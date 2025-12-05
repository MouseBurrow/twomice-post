CREATE OR REPLACE FUNCTION post.create_post(
    token TEXT,
    new_post_content TEXT,
    new_image_url TEXT
)
    RETURNS TABLE
            (
                user_id      UUID,
                post_content TEXT,
                image_url    TEXT,
                created_at   TIMESTAMP
            )
    LANGUAGE plpgsql
AS
$$
DECLARE
    uid UUID;
BEGIN
    SELECT user_id
    INTO uid
    FROM auth.sessions
    WHERE session_token = token;

    IF uid IS NULL THEN
        RAISE EXCEPTION 'Invalid token' USING ERRCODE = '23502';
    END IF;

    -- Insert new post and return full row
    RETURN QUERY
        INSERT INTO post.posts (userid, content, image_url)
            VALUES (uid, new_post_content, new_image_url)
            RETURNING userid, content, image_url, created_at;
END;
$$;
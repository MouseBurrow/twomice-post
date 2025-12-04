CREATE OR REPLACE FUNCTION post.create_post(
    new_user_id UUID,
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
BEGIN
    -- Insert the new post and return the inserted row
    RETURN QUERY
        INSERT INTO post.posts (userid, content, image_url)
            VALUES (new_user_id, new_post_content, new_image_url)
            RETURNING created_at, id;
END;
$$;
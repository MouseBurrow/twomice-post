CREATE OR REPLACE FUNCTION create_post(
    new_user_id UUID,
    new_post_content TEXT,
    new_image_url TEXT
)
    RETURNS TABLE
            (
                id         UUID,
                created_at TIMESTAMP
            )
    LANGUAGE plpgsql
AS
$$
BEGIN
    -- Insert the new post and return the inserted row
    RETURN QUERY
        INSERT INTO posts (user_id, content, image_url)
            VALUES (new_user_id, new_post_content, new_image_url)
            RETURNING id, created_at;
END;
$$;
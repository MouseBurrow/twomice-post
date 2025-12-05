CREATE OR REPLACE FUNCTION create_post(
    token TEXT,
    new_post_content TEXT,
    new_image_url TEXT
)
    RETURNS TIMESTAMPTZ
    LANGUAGE plpgsql
AS
$$
DECLARE
    existing_user_id UUID;
    new_created_at   TIMESTAMPTZ;
BEGIN
    SELECT account_id
    INTO existing_user_id
    FROM auth.sessions
    WHERE session_token = token;

    IF existing_user_id IS NULL THEN
        RAISE EXCEPTION 'Invalid token' USING ERRCODE = '23502';
    END IF;

    -- Insert new post and return full row
    INSERT INTO posts as p (user_id, content, image_url)
    VALUES (existing_user_id, new_post_content, new_image_url)
    RETURNING created_at INTO new_created_at;

    RETURN new_created_at;
END;
$$;
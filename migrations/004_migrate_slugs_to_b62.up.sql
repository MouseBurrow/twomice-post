-- Convert existing post slugs to base62-encoded snowflake IDs.

CREATE OR REPLACE FUNCTION b62_encode(n BIGINT) RETURNS TEXT AS $$
DECLARE
    chars CONSTANT TEXT := '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
    result TEXT := '';
    val BIGINT := n;
BEGIN
    IF val = 0 THEN
        RETURN '0';
    END IF;
    WHILE val > 0 LOOP
        result := substr(chars, (val % 62)::int + 1, 1) || result;
        val := val / 62;
    END LOOP;
    RETURN result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

UPDATE posts SET slug = b62_encode(id)
WHERE slug !~ '^[0-9A-Za-z]+$'
   OR slug = '';

DROP FUNCTION b62_encode(BIGINT);

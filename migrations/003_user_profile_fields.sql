ALTER TABLE users
    ADD COLUMN IF NOT EXISTS gender SMALLINT;

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS nick_name VARCHAR(100);

UPDATE users
SET nick_name = real_name
WHERE nick_name IS NULL;

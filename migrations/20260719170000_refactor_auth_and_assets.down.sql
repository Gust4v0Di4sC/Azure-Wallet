DROP INDEX IF EXISTS assets_user_ticker_idx;
DROP INDEX IF EXISTS assets_user_id_idx;

ALTER TABLE assets
    DROP COLUMN currency,
    DROP COLUMN current_price,
    DROP COLUMN average_price,
    DROP COLUMN quantity,
    DROP COLUMN asset_class,
    DROP COLUMN ticker,
    DROP COLUMN user_id,
    ADD COLUMN unit_value DOUBLE PRECISION NOT NULL DEFAULT 0;

ALTER TABLE assets ADD CONSTRAINT assets_name_key UNIQUE (name);

ALTER TABLE users RENAME COLUMN email TO username;

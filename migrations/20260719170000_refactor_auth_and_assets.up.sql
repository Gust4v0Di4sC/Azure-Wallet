ALTER TABLE users RENAME COLUMN username TO email;

DELETE FROM assets;

ALTER TABLE assets DROP CONSTRAINT IF EXISTS assets_name_key;
ALTER TABLE assets DROP COLUMN unit_value;

ALTER TABLE assets
    ADD COLUMN user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN ticker TEXT NOT NULL,
    ADD COLUMN asset_class TEXT NOT NULL,
    ADD COLUMN quantity NUMERIC(20, 8) NOT NULL,
    ADD COLUMN average_price NUMERIC(20, 2) NOT NULL,
    ADD COLUMN current_price NUMERIC(20, 2) NOT NULL,
    ADD COLUMN currency TEXT NOT NULL DEFAULT 'USD';

CREATE INDEX assets_user_id_idx ON assets(user_id);
CREATE UNIQUE INDEX assets_user_ticker_idx ON assets(user_id, ticker);

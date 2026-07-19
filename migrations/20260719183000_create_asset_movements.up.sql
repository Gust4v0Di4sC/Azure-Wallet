CREATE TABLE IF NOT EXISTS asset_movements (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_id BIGINT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    ticker TEXT NOT NULL,
    asset_name TEXT NOT NULL,
    movement_type TEXT NOT NULL,
    quantity NUMERIC(20, 8) NOT NULL,
    unit_price NUMERIC(20, 2) NOT NULL,
    currency TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT asset_movements_type_check CHECK (movement_type IN ('entry', 'exit'))
);

CREATE INDEX asset_movements_user_created_idx ON asset_movements(user_id, created_at DESC);

INSERT INTO users (email, password_hash)
VALUES ('user@example.com', 'hash');

INSERT INTO assets (
    user_id,
    name,
    ticker,
    asset_class,
    quantity,
    average_price,
    current_price,
    currency
)
VALUES (1, 'Bitcoin', 'BTC', 'crypto', 2.0, 5.00, 10.00, 'USD');

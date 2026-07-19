<p align="center">
  <img src="src/assets/AzureWLogo.svg" alt="Azure Wallet logo" width="140">
</p>

# Azure Wallet

Digital wallet built with Rust, Axum, Askama, SQLx and PostgreSQL.

## Requirements

- Docker and Docker Compose
- Rust 1.97+ only if you want to run the app locally without Docker
- SQLx CLI only if you want to run migrations manually

## Run Everything With Docker

This is the recommended setup for a new machine.

1. Create a local `.env` file:

```powershell
Copy-Item .env.example .env
```

2. Replace `AUTH_COOKIE_KEY` and `JWT_SECRET` in `.env` with long random values.

PowerShell example:

```powershell
[Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Maximum 256 }))
```

Run the command twice and use different values for `AUTH_COOKIE_KEY` and `JWT_SECRET`.

3. Start the full stack:

```powershell
docker compose up --build
```

4. Open the app:

```text
http://localhost:3000/login
```

The app waits for PostgreSQL to be healthy and runs database migrations automatically on startup.

To stop:

```powershell
Ctrl+C
docker compose down
```

To reset the database volume:

```powershell
docker compose down -v
```

## Run Locally With Cargo

Use this flow when you are developing Rust code directly on your machine.

1. Start only PostgreSQL:

```powershell
docker compose up -d db
```

2. Create `.env` from `.env.example` and set:

```env
DATABASE_URL=postgres://wallet:wallet@localhost:55432/wallet_live
COOKIE_SECURE=false
```

3. Start the app:

```powershell
cargo run
```

The server starts at:

```text
http://localhost:3000
```

Login page:

```text
http://localhost:3000/login
```

## Authentication

- Register and login are separate flows.
- Passwords are hashed with `bcrypt`.
- Session token is stored in the `auth_token` cookie.
- Cookies are `HttpOnly`, `SameSite=Lax` and encrypted with `PrivateCookieJar`.
- Secrets must stay in `.env`, which is ignored by Git.

## Wallet Features

Each asset belongs to an authenticated user and includes:

- `name`
- `ticker`
- `asset_class`
- `quantity`
- `average_price`
- `current_price`
- `currency`

Money and quantities use `rust_decimal::Decimal`.

The dashboard includes:

- total portfolio value;
- local currency selector;
- asset list;
- entry and exit history with green/red indicators.

Currency conversion uses a local static conversion table in the application code. It is not live market data.

## Tests

Compile tests without connecting to PostgreSQL:

```powershell
cargo test --no-run
```

Run the SQLx-backed tests with PostgreSQL available:

```powershell
docker compose up -d db
cargo test
```

## Useful Commands

Format and check:

```powershell
cargo fmt
cargo check
```

View logs when using Docker:

```powershell
docker compose logs -f app
```

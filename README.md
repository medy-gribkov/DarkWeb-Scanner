# SporeSec DarkWeb Scanner

This repository contains a Rust-based API server and a NodeJS frontend for the SporeSec DarkWeb Scanner.

Quick fixes applied:
- `Cargo.toml` edition set to `2021` to ensure compatibility with current Rust toolchains.
- `src/db.rs` SQLite token persistence fixed to store `exp` and `is_token_revoked` logic corrected.
- Added usage and Docker instructions.

Requirements
- Docker & docker-compose
- Rust toolchain (for local builds)
- Node (for frontend local dev)

Running with Docker

1. Build and start services:

```
docker-compose up --build
```

This will start:
- `tor` (dperson/torproxy) exposing a socks5 proxy on port 9050
- `sporesec-api` (Rust API) on port 8080
- `frontend` (Node) on port 3000

Environment
- `ADMIN_API_KEY` - admin key for token issuance (Strictly required)
- `JWT_SECRET` - secret for signing tokens (Strictly required)
- `SPORE_SIGNATURE` / `NEXT_PUBLIC_SPORE_SIGNATURE` - shared signature between frontend and backend

CLI mode
You can run the binary directly to perform CLI scans:

```
cargo run -- --cli "query" --concurrency 4 --output ./output.json
```

Notes
- The project relies on a local Tor proxy (9050). The docker-compose includes one for convenience.
- For production, set `DATABASE_URL` to use Postgres/Neon.


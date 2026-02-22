# DarkWeb-Scanner

[![CI Guardrails](https://github.com/medy-gribkov/DarkWeb-Scanner/actions/workflows/ci.yml/badge.svg)](https://github.com/medy-gribkov/DarkWeb-Scanner/actions/workflows/ci.yml)

**Industrial-grade dark web scanner built with Rust and Node.js.**

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Node.js](https://img.shields.io/badge/Node.js-339933?style=for-the-badge&logo=nodedotjs&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white)
![Tor](https://img.shields.io/badge/Tor-7D4698?style=for-the-badge&logo=torproject&logoColor=white)
![License](https://img.shields.io/badge/License-Restrictive-red?style=for-the-badge)

DarkWeb-Scanner orchestrates multi-threaded scraping through the Tor network using a high-performance Rust engine. It pairs a compiled Rust API backend with a Node.js frontend dashboard, all deployable via Docker Compose with an integrated Tor proxy.

---

## Features

- **Multi-threaded Rust engine** for concurrent dark web scraping with configurable concurrency levels
- **Tor network integration** via SOCKS5 proxy, bundled in the Docker stack
- **REST API** with JWT authentication and admin token issuance
- **Web frontend** built with Node.js for scan management and result viewing
- **CLI mode** for headless, scriptable scans with JSON output
- **Docker Compose deployment** for single-command setup of all services (API, frontend, Tor proxy)
- **SQLite/Postgres persistence** with token management and revocation support

## Architecture

```
+-------------+       +--------------+       +-----------+
|  Frontend   | ----> |   Rust API   | ----> | Tor Proxy |  ----> .onion targets
|  (Node.js)  |       |  (port 8080) |       | (port 9050)|
|  port 3000  |       +--------------+       +-----------+
+-------------+              |
                             v
                       +-----------+
                       |  Database  |
                       | SQLite/PG  |
                       +-----------+
```

The Rust core handles all scraping logic, authentication, and database operations. The Node.js frontend provides a web interface for managing scans. Tor proxy routes all outbound traffic through the Tor network.

## Quick Start

### Requirements

- Docker and Docker Compose
- Rust toolchain (for local builds only)
- Node.js (for frontend local dev only)

### Docker Compose (recommended)

1. Set required environment variables:

```bash
export ADMIN_API_KEY="your-admin-key"
export JWT_SECRET="your-jwt-secret"
```

2. Build and start all services:

```bash
docker-compose up --build
```

This starts three containers:

| Service | Description | Port |
|---------|-------------|------|
| `tor` | Tor SOCKS5 proxy (dperson/torproxy) | 9050 |
| `sporesec-api` | Rust API server | 8080 |
| `frontend` | Node.js web dashboard | 3000 |

### CLI Mode

Run scans directly from the command line without the web frontend:

```bash
cargo run -- --cli "query" --concurrency 4 --output ./output.json
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ADMIN_API_KEY` | Yes | Admin key for token issuance |
| `JWT_SECRET` | Yes | Secret for signing JWT tokens |
| `SPORE_SIGNATURE` | No | Shared signature between frontend and backend |
| `NEXT_PUBLIC_SPORE_SIGNATURE` | No | Frontend-side shared signature |
| `DATABASE_URL` | No | Database connection string (defaults to SQLite, use Postgres/Neon for production) |

## Documentation

- [API Reference](API.md) -- endpoint definitions, authentication, request/response formats
- [Deployment Guide](DEPLOYMENT.md) -- production setup, scaling, environment configuration
- [Implementation Details](IMPLEMENTATION.md) -- architecture decisions, module breakdown, data flow

## Disclaimer

This tool is designed for **authorized security research and penetration testing only**. Use of this software to access systems, networks, or data without explicit written authorization is illegal and unethical. The authors assume no liability for misuse. Always obtain proper authorization before conducting any security assessments.

## License

This project is distributed under a restrictive license. See [LICENSE_RESTRICTIVE.md](LICENSE_RESTRICTIVE.md) for terms.

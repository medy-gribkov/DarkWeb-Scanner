# SPORESEC DarkWeb Scanner API

Production-ready Sovereign Intelligence Engine API for Darknet discovery, built with Rust + Actix-web, backed by Neon Postgres (with SQLite fallback).

## Features

- **Postgres-first architecture**  with SQLite fallback for CLI
- **JWT authentication** with token lifecycle management (`jti`, revocation)
- **Role-based access** (anonymous, authenticated, paid tiers)
- **Improved scanning** with per-link timeouts, exponential backoff, better categorization
- **Type-safe database layer** supporting both Postgres and SQLite
- **Prometheus metrics** for observability
- **Server-to-server auth** via Admin API Key and Bearer tokens

## Quick Start (Local)

### Prerequisites
- Rust 1.70+
- Docker & Docker Compose
- Tor proxy (via docker-compose)

### Local Development

```bash
# 1. Clone and install deps
git clone https://github.com/Spore-Sec/SporeSec-DarkWeb-Scanner
cd SporeSec-DarkWeb-Scanner
cargo build --release

# 2. Run with docker-compose (includes Tor + API + frontend demo)
docker-compose up --build

# 3. API is available at http://localhost:8080
# Frontend demo at http://localhost:3000
```

### Environment Variables (Local)
```bash
# Leave DATABASE_URL empty to use SQLite
export SPORE_SIGNATURE="supersecret"
export ADMIN_API_KEY="admin-key-here"
export JWT_SECRET="your-secret-key-here"
export FRONTEND_DOMAIN="http://localhost:3000"
export RUST_LOG="info"
```

### CLI Mode
```bash
cargo run --release -- --cli "test query" --output results.json
```

## API Endpoints

### 1. Issue Token (Server-to-Server)
**POST /v1/token**

Required headers:
- `X-Admin-Key: {ADMIN_API_KEY}`

Body:
```json
{
  "client_id": "user@your-site.com"
}
```

Response:
```json
{
  "token": "eyJ...",
  "expires_in": 3600,
  "jti": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### 2. Scan (Requires Signature + Optional Bearer Token)
**GET /v1/scan?query={query}**

Headers:
- `X-Spore-Signature: {SPORE_SIGNATURE}` (required)
- `Authorization: Bearer {token}` (optional, enables enhanced results)

Response (anonymous):
```json
{
  "results": [
    {
      "title": "onion site",
      "status": "Online",
      "signup_required": true
    }
  ],
  "user": { "authenticated": false },
  "price": 8.99
}
```

Response (authenticated, unpaid):
```json
{
  "results": [
    {
      "title": "onion site",
      "status": "Online",
      "link": "example.onion",
      "category": "market",
      "purchase_prompt": {"price": 8.99, "endpoint": "/v1/purchase"}
    }
  ],
  "user": { "authenticated": true, "paid": false, "client_id": "user@..." },
  "price": 8.99
}
```

Response (authenticated, paid):
```json
{
  "results": [
    {
      "raw_onion": "...",
      "title": "...",
      "link": "...",
      "category": "...",
      "status": "Online",
      "discovered_at": "2024-01-01T00:00:00.000Z",
      "source": "..."
    }
  ],
  "user": { "authenticated": true, "paid": true, "client_id": "user@..." },
  "price": 8.99
}
```

---

### 3. Purchase (Requires Valid Bearer Token)
**POST /v1/purchase**

Headers:
- `Authorization: Bearer {token}`

Response:
```json
{
  "status": "paid",
  "client_id": "user@...",
  "price": 8.99
}
```

---

### 4. Revoke Token (Admin Only)
**POST /v1/revoke**

Headers:
- `X-Admin-Key: {ADMIN_API_KEY}`

Body:
```json
{
  "jti": "550e8400-e29b-41d4-a716-446655440000"
}
```

Response:
```json
{
  "status": "revoked",
  "jti": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### 5. Metrics
**GET /metrics**

Prometheus-format metrics.

---

## Integration Guide (TypeScript + Drizzle)

### 1. Issue Token (Your Next.js Backend)

```typescript
// pages/api/auth/scan-token.ts
import { verifyAuth } from '@/lib/auth';

export default async function handler(req, res) {
  const user = await verifyAuth(req);
  if (!user) return res.status(401).json({ error: 'Unauthorized' });

  const response = await fetch('http://api:8080/v1/token', {
    method: 'POST',
    headers: {
      'X-Admin-Key': process.env.SPORESEC_ADMIN_KEY!,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ client_id: user.email }),
  });

  const data = await response.json();
  res.status(200).json({ token: data.token, jti: data.jti });
}
```

### 2. Mark as Paid (After Stripe Webhook)

```typescript
// pages/api/webhooks/stripe.ts
import Stripe from 'stripe';
import { db } from '@/db';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

export default async function handler(req, res) {
  const event = stripe.webhooks.constructEvent(req.body, req.headers['stripe-signature']!);

  if (event.type === 'payment_intent.succeeded') {
    const customerId = event.data.object.metadata.customer_id;
    const user = await db.user.findUnique({ where: { id: customerId } });

    // Call API to mark paid
    await fetch('http://api:8080/v1/purchase', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${user.scanToken}`,
        'X-Spore-Signature': process.env.SPORE_SIGNATURE!,
      },
    });

    res.status(200).json({ received: true });
  }
}
```

### 3. Call Scan API (Browser)

```typescript
// lib/scan.ts
export async function scanOnions(query: string, token?: string) {
  const url = new URL('http://api:8080/v1/scan');
  url.searchParams.set('query', query);

  const headers: Record<string, string> = {
    'X-Spore-Signature': process.env.NEXT_PUBLIC_SPORE_SIGNATURE || 'supersecret',
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(url, { headers });
  return res.json();
}
```

---

## Deployment (Railway/Neon + Fly.io)

### 1. Create Neon Postgres Database
```bash
# Get DATABASE_URL from Neon console
# Example: postgresql://user:pass@ep-xxx.neon.tech/sporesec
```

### 2. Environment Variables
```bash
DATABASE_URL="postgresql://..."
SPORE_SIGNATURE="your-secret"
ADMIN_API_KEY="your-admin-key"
JWT_SECRET="your-jwt-secret"
FRONTEND_DOMAIN="https://your-site.example.com"
RUST_LOG="info"
```

### 3. Deploy to Fly.io
```bash
flyctl launch
flyctl secrets set DATABASE_URL="..." ADMIN_API_KEY="..." JWT_SECRET="..."
flyctl deploy
```

### 4. Integrate with Your Next.js App
In your `.env.local`:
```
SPORESEC_API_URL=https://your-api.fly.dev
SPORESEC_ADMIN_KEY=your-admin-key
NEXT_PUBLIC_SPORE_SIGNATURE=your-signature
```

---

## Database Schema

### Clients Table
```sql
CREATE TABLE clients (
  id TEXT PRIMARY KEY,
  paid BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Token Metadata Table
```sql
CREATE TABLE token_metadata (
  jti TEXT PRIMARY KEY,
  client_id TEXT,
  issued_at TIMESTAMP,
  exp BIGINT,
  revoked_at TIMESTAMP DEFAULT NULL
);
```

---

## Security Best Practices

1. **Store secrets in a vault** (AWS Secrets Manager, Vault, or 1Password)
2. **Use TLS at the edge** (reverse proxy)
3. **Rate-limit by client_id** to prevent abuse
4. **Rotate JWT_SECRET regularly** (use `/v1/rotate-secret` or re-deploy)
5. **Monitor revoked tokens** and clean up expired metadata monthly
6. **Restrict ADMIN_API_KEY** to server-to-server calls only
7. **Never expose Tor control port** publicly

---

## Monitoring & Logs

View Prometheus metrics:
```bash
curl http://localhost:8080/metrics
```

Query logs (with env_logger):
```bash
RUST_LOG=debug cargo run
```

---

## Troubleshooting

### "DATABASE_URL not set; using SQLite"
- This is **expected for local dev**. Postgres is optional.
- To use Postgres locally, set `DATABASE_URL=postgresql://localhost/sporesec`

### Token Expired
- Tokens are valid for 1 hour. Issue a new token before expiration.

### Revocation Not Working
- Ensure the `jti` is correct. Check token_metadata table for existence.

---

## License

MIT


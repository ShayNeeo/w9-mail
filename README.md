# W9 Mail

API-first transactional email service for W9 Labs.

## Tech Stack

- **Backend**: Rust + Axum + SurrealDB + lettre (SMTP)
- **Frontend**: Leptos (Full-stack SSR)
- **SMTP**: Microsoft Outlook (outlook.com:587 STARTTLS)

## Features

- Email composition and sending via SMTP
- Account and alias management
- API token generation for programmatic access
- HTML email support with inline images
- W9 Mail branded email templates

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/send` | Send email |
| GET/POST | `/api/accounts` | Manage email accounts |
| GET/POST | `/api/aliases` | Manage aliases |
| GET/POST | `/api/api-tokens` | API token management |
| GET | `/api/inbox` | Inbox (placeholder) |

## Quick Start

```bash
cargo run --package w9-mail-server
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SurrealDB connection | `memory` |
| `SMTP_HOST` | SMTP server | `smtp-mail.outlook.com` |
| `SMTP_PORT` | SMTP port | `587` |
| `W9_DB_URL` | W9 DB OAuth URL | `https://db.w9.nu` |
| `PORT` | Server port | `10106` |

## Deployment

```bash
docker-compose up -d
```

Access at: `https://mail.w9.nu`

## License

GPL v3.0

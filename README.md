# W9 Mail

W9 Mail is an email service layer that sits between your applications and Microsoft SMTP/IMAP/POP3. It provides a RESTful API and web interface for managing email accounts, aliases, and sending emails programmatically.

## Table of Contents

- [Overview](#overview)
- [For Website Users](#for-website-users)
  - [Getting Started](#getting-started)
  - [Account Management](#account-management)
  - [Sending Emails](#sending-emails)
  - [API Tokens](#api-tokens)
  - [User Roles](#user-roles)
- [For Developers / Deployment](#for-developers--deployment)
  - [Prerequisites](#prerequisites)
  - [Quick Installation](#quick-installation)
  - [Configuration](#configuration)
  - [Environment Variables](#environment-variables)
  - [Service Management](#service-management)
  - [API Integration](#api-integration)
  - [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Overview

W9 Mail provides:
- **Web Interface**: Manage email accounts, aliases, and users through a Next.js frontend
- **RESTful API**: Send emails and manage resources programmatically
- **Microsoft Integration**: Seamless connection to Microsoft 365/Azure SMTP/IMAP/POP3
- **Role-Based Access**: Admin, Dev, and User roles with granular permissions
- **API Token Authentication**: Long-lived tokens for service-to-service communication
- **Asset Ownership**: Accounts and aliases can be owned by users with public/private visibility

**Tech Stack:**
- **Backend**: Rust (Axum + SQLx)
- **Frontend**: Next.js/TypeScript
- **Database**: SQLite
- **Deployment**: Systemd service + Nginx reverse proxy

---

## For Website Users

### Getting Started

1. **Sign Up**: Visit the signup page and create an account with your email
2. **Verify Email**: Check your email for a verification link (expires in 30 minutes)
3. **Login**: After verification, log in with your credentials
4. **Change Password**: If required, update your password on first login

### Account Management

#### Viewing Accounts and Aliases

- Navigate to the **Manage** page (`/manage`)
- View all email accounts and aliases you have access to
- Accounts/aliases show:
  - Email address
  - Display name
  - Active status
  - Owner (if owned by someone)
  - Public/Private status

#### Creating Accounts (Dev/Admin Only)

1. Go to **Manage** → **Accounts** section
2. Click **Add Account**
3. Fill in:
   - **Email**: Microsoft email address
   - **Display Name**: Friendly name for the account
   - **Password**: Microsoft account password or app password
   - **Public**: Check to make visible to other users
4. Click **Create Account**

#### Creating Aliases (Dev/Admin Only)

1. Go to **Manage** → **Aliases** section
2. Click **Add Alias**
3. Fill in:
   - **Account**: Select the account this alias belongs to
   - **Alias Email**: The alias email address
   - **Display Name**: Optional friendly name
   - **Public**: Check to make visible to other users
4. Click **Create Alias**

#### Managing Your Owned Assets

If you own an account or alias, you can:
- **Toggle Public/Private**: Make it visible or hidden to other users
- **Delete**: Remove the account/alias (only if you own it or are an admin)
- **View Usage**: See when it was created and last used

### Sending Emails

#### Using the Web Interface

1. Go to the **Compose** page (`/`)
2. Select a **From** address (you'll see public accounts/aliases or all if you're admin/dev)
3. Enter:
   - **To**: Recipient email address
   - **Subject**: Email subject
   - **Body**: Email content (supports HTML)
   - **CC/BCC**: Optional additional recipients
4. Click **Send Email**

#### Using the API

See the [API Documentation](#api-integration) section for programmatic access.

### API Tokens

API tokens are long-lived tokens for authenticating API requests (unlike login tokens which expire after 12 hours).

#### Creating an API Token

1. Go to **Profile** (`/profile`)
2. Scroll to **API Tokens** section
3. Optionally enter a name for the token (e.g., "Production API", "Development")
4. Click **Create API Token**
5. **⚠️ IMPORTANT**: Copy the token immediately - it's only shown once!
6. Click **I've Saved It** to dismiss the token display

#### Managing API Tokens

- View all your tokens with creation date and last used timestamp
- Delete tokens you no longer need
- Tokens work forever until deleted (no expiration)

#### Using API Tokens

Include the token in API requests:
```bash
curl -H "Authorization: Bearer YOUR_API_TOKEN" \
     https://w9.nu/api/accounts
```

### User Roles

#### Normal User
- Can read API documentation
- Cannot use the API endpoints
- Can view public accounts/aliases in compose
- Can apply to become a Dev user (email `hi@w9.nu`)

#### Dev User
- Can use all API endpoints
- Can create accounts and aliases
- Can manage their own accounts/aliases
- Cannot manage other users
- Cannot manage default sender

#### Admin User
- Full access to all features
- Can manage all users, accounts, and aliases
- Can set account/alias ownership
- Can configure default sender
- Can change user roles

---

## For Developers / Deployment

### Prerequisites

- Docker and Docker Compose installed on your VPS
- GitHub Container Registry (GHCR) access configured
- Microsoft 365/Azure app credentials:
  - Client ID
  - Client Secret
  - Tenant ID

### Docker Deployment

This project uses **Docker with CI/CD** for deployment. Images are automatically built and pushed to GitHub Container Registry on every push to `main`.

#### Quick Start

1. **Set up deployment on your VPS:**
   
   Clone the repository and configure environment variables:
   
   ```bash
   # On your VPS
   git clone https://github.com/your-username/w9-mail.git
   cd w9-mail
   
   # Configure environment variables
   cp .env.example .env
   nano .env
   ```

2. **Login to GitHub Container Registry (if using private images):**
   ```bash
   echo $GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin
   ```

3. **Deploy with Docker Compose:**
   ```bash
   # Pull latest images
   docker-compose pull
   
   # Start services
   docker-compose up -d
   ```

4. **View logs:**
   ```bash
   docker-compose logs -f w9-mail-backend
   ```

5. **Access the web interface:**
   Visit `https://your-domain` in your browser

### CI/CD

The project includes a GitHub Actions workflow (`.github/workflows/docker-build.yml`) that:
- Builds both backend and frontend Docker images on every push to `main`
- Pushes images to `ghcr.io/<your-username>/w9-mail-backend` and `w9-mail-frontend`
- Tags images with `latest` and commit SHA

**Watchtower** (included in docker-compose.yml) automatically updates containers when new images are pushed.

### Configuration

Configuration is managed via environment variables in the `.env` file. Copy `.env.example` to `.env` and fill in your values.

#### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HOST` | Backend bind address | `0.0.0.0` | No |
| `PORT` | Backend port | `8080` | No |
| `BASE_URL` | Public base URL | `https://w9.nu` | Yes |
| `APP_WEB_BASE_URL` | Frontend base URL | Same as `BASE_URL` | No |
| `DATABASE_PATH` | SQLite database path | `/opt/w9-mail/data/w9mail.db` | No |
| `JWT_SECRET` | Secret for JWT tokens | `change-me-in-production` | **Yes** (change!) |
| `MICROSOFT_CLIENT_ID` | Azure App Client ID | - | **Yes** |
| `MICROSOFT_CLIENT_SECRET_ID` | Azure Client Secret | - | **Yes** |
| `MICROSOFT_CLIENT_VALUE` | Optional custom value | - | No |
| `MICROSOFT_TENANT_ID` | Azure Tenant/Directory ID | - | **Yes** |
| `MICROSOFT_REDIRECT_URI` | OAuth redirect URL | `https://w9.nu/api/auth/callback` | No |
| `MICROSOFT_SCOPE` | OAuth scopes | `https://outlook.office.com/IMAP.AccessAsUser.All https://outlook.office.com/SMTP.Send` | No |
| `TURNSTILE_SECRET_KEY` | Cloudflare Turnstile secret | - | No |

> **Security Note**: Always change `JWT_SECRET` to a strong random string in production!

#### Updating Configuration

1. Edit `.env` file:
   ```bash
   nano .env
   ```

2. Restart the containers:
   ```bash
   docker-compose restart w9-mail-backend w9-mail-frontend
   ```

### Service Management

#### Check Service Status
```bash
docker-compose ps
```

#### View Logs
```bash
# Follow logs in real-time
docker-compose logs -f w9-mail-backend

# View recent logs
docker-compose logs --tail=100 w9-mail-backend
```

#### Restart Service
```bash
docker-compose restart w9-mail-backend
```

#### Update to Latest Version
```bash
# Pull latest images
docker-compose pull

# Restart with new images
docker-compose up -d
```

> **Note**: Watchtower automatically updates containers when new images are pushed to the registry. Manual updates are only needed if you want to update immediately.

#### Check Service Status
```bash
systemctl status w9-mail
```

#### View Logs
```bash
# Follow logs in real-time
journalctl -u w9-mail -f

# View recent logs
journalctl -u w9-mail -n 100
```

#### Restart Service
```bash
sudo systemctl restart w9-mail
```

#### Stop/Start Service
```bash
sudo systemctl stop w9-mail
sudo systemctl start w9-mail
```

#### Redeploy After Code Changes
```bash
cd /path/to/w9-mail
git pull
sudo -E ./install.sh
```

### API Integration

#### Authentication

W9 Mail supports two authentication methods:

1. **JWT Tokens** (from login, expire after 12 hours):
   ```bash
   # Login to get token
   curl -X POST https://w9.nu/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"user@example.com","password":"password"}'
   ```

2. **API Tokens** (long-lived, created in profile page):
   ```bash
   # Use API token directly
   curl -H "Authorization: Bearer YOUR_API_TOKEN" \
     https://w9.nu/api/accounts
   ```

#### Common Endpoints

**Send Email:**
```bash
POST /api/send
Content-Type: application/json
Authorization: Bearer YOUR_TOKEN

{
  "from": "sender@example.com",
  "to": "recipient@example.com",
  "subject": "Hello",
  "body": "Email content",
  "isHtml": true,
  "cc": "optional@example.com",
  "bcc": "optional@example.com"
}
```

**List Accounts:**
```bash
GET /api/accounts
Authorization: Bearer YOUR_TOKEN
```

**List Aliases:**
```bash
GET /api/aliases
Authorization: Bearer YOUR_TOKEN
```

**Get Default Sender:**
```bash
GET /api/settings/default-sender
Authorization: Bearer YOUR_TOKEN
```

**Set Default Sender:**
```bash
PUT /api/settings/default-sender
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

{
  "senderType": "account",
  "senderId": "account-id"
}
```

#### Full API Documentation

Visit `/docs` in the web interface for complete API documentation with examples.

### Troubleshooting

#### Email Send TLS Errors
- **Problem**: TLS handshake fails when sending emails
- **Solution**: 
  - Verify Microsoft credentials are correct
  - Ensure account allows SMTP access
  - Use app password if 2FA is enabled
  - Check account password hasn't expired

#### Account Status Not Updating
- **Problem**: Frontend shows incorrect account status
- **Solution**: 
  - Verify `NEXT_PUBLIC_API_URL` is set correctly
  - Check browser console for API errors
  - Ensure backend is running and accessible

#### Database Issues
- **Problem**: Database errors or permission denied
- **Solution**:
  - Verify `DATABASE_PATH` exists and is writable
  - Check service user permissions: `ls -la /opt/w9-mail/data`
  - Ensure SQLite is installed: `sudo apt install sqlite3`

#### nginx SSL Warnings
- **Problem**: SSL certificate warnings
- **Solution**:
  - If using Cloudflare: Use Cloudflare origin certificates
  - Otherwise: Install Let's Encrypt certificates
  - Self-signed certs are generated as fallback

#### Service Won't Start
- **Problem**: `systemctl status w9-mail` shows failed
- **Solution**:
  - Check logs: `journalctl -u w9-mail -n 50`
  - Verify environment variables in `/etc/default/w9-mail`
  - Check port is not in use: `sudo lsof -i :8080`
  - Verify database path is accessible

#### API Token Not Working
- **Problem**: API requests with token return 401
- **Solution**:
  - Verify token is correct (no extra spaces)
  - Check token wasn't deleted
  - Ensure `Authorization: Bearer TOKEN` header format
  - Verify user account is still active

---

## Architecture

### Backend Structure
- `backend/src/main.rs` - Application entry point, routes, database setup
- `backend/src/auth.rs` - Authentication, JWT, API tokens, user management
- `backend/src/handlers.rs` - API endpoint handlers
- `backend/src/email.rs` - Email sending service (Microsoft SMTP)
- `backend/src/mailer.rs` - Mailer configuration and default sender

### Frontend Structure
- `frontend/app/` - Next.js App Router pages
  - `page.tsx` - Compose email page
  - `manage/page.tsx` - Account/alias/user management
  - `profile/page.tsx` - User profile and API tokens
  - `docs/page.tsx` - API documentation
  - `login/page.tsx` - Login page
  - `signup/` - Registration flow

### Database Schema
- `users` - User accounts with roles
- `accounts` - Email accounts (Microsoft credentials)
- `aliases` - Email aliases linked to accounts
- `api_tokens` - API authentication tokens
- `default_sender` - Default account/alias for transactional emails
- `pending_users` - Email verification tokens
- `password_reset_tokens` - Password reset tokens

### Security Features
- Argon2 password hashing
- JWT token authentication (12-hour expiration)
- API token authentication (no expiration, user-managed)
- Role-based access control (Admin, Dev, User)
- Asset ownership and public/private visibility
- Cloudflare Turnstile integration (optional)

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/xyz`
3. Make your changes
4. Test thoroughly
5. Run linters/formatters:
   - Backend: `cargo fmt && cargo clippy`
   - Frontend: `npm run lint`
6. Commit with descriptive messages
7. Push and open a pull request

---

## License

Licensed under GNU General Public License v3.0. See [LICENSE](LICENSE) file for details.

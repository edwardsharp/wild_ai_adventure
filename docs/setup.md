# Quick Database Setup Guide

This guide will help you set up PostgreSQL and test the invite code functionality.

## 1. Install and Start PostgreSQL

### macOS (using Homebrew)

```bash
brew install postgresql@15
brew services start postgresql@15
```

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### Windows

Download and install from: https://www.postgresql.org/download/windows/

## 2. Create Database and User

```bash
# Connect to PostgreSQL as superuser
sudo -u postgres psql

# Or on macOS with Homebrew:
psql postgres

# Create a new user and database
CREATE USER webauthn_user WITH PASSWORD 'secure_password123';
CREATE DATABASE webauthn_db OWNER webauthn_user;
GRANT ALL PRIVILEGES ON DATABASE webauthn_db TO webauthn_user;
\q
```

## 3. Configure Environment

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env file with your database credentials
# DATABASE_URL=postgresql://webauthn_user:secure_password123@localhost:5432/webauthn_db
```

## 4. Test Database Connection

```bash
# Test the CLI tool (this will also run migrations)
cargo run --bin cli users stats
```

If successful, you should see:

```
Invite Code Statistics:
  Total codes: 0
  Active codes: 0
  Used codes: 0
  Unused codes: 0
```

## 5. Generate Your First Invite Code

```bash
# Generate a single invite code
cargo run --bin cli users generate-invite

# Generate multiple codes
cargo run --bin cli users generate-invite --count 5

# Generate longer codes
cargo run --bin cli users generate-invite --length 12
```

## 6. View Invite Codes

```bash
# List all invite codes
cargo run --bin cli users list-invites

# List only active/unused codes
cargo run --bin cli users list-invites --active-only
```

## 7. Start the Web Server

```bash
# Using the development script (recommended)
./scripts/start_dev.sh

# Or manually
cargo run
```

## Troubleshooting

### Error: "password authentication failed"

- Check your username and password in the .env file
- Ensure the PostgreSQL user was created correctly

### Error: "database does not exist"

- Make sure you created the database: `CREATE DATABASE webauthn_db;`

### Error: "connection refused"

- Ensure PostgreSQL is running: `brew services start postgresql@15` (macOS) or `sudo systemctl start postgresql` (Linux)

### Error: "relative URL without a base"

- This was a bug that has been fixed. Update to the latest version of the code.

## Quick Test Sequence

1. Generate an invite code:

   ```bash
   cargo run --bin cli users generate-invite
   ```

2. Note the generated code (e.g., "ABC12345")

3. Start the server:

   ```bash
   ./scripts/start_dev.sh
   ```

4. Open http://localhost:8080 in your browser

5. Try to register:

   - Username: `testuser`
   - Invite Code: `ABC12345` (use the code you generated)
   - Complete WebAuthn registration with your device

6. Verify the invite code was used:
   ```bash
   cargo run --bin cli users list-invites
   ```

The code should now show as used!

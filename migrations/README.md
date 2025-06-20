# Database Migrations

This directory contains the database schema migrations for the WebAuthn authentication server.

## Directory Structure

```
migrations/
├── README.md                    # This file
├── 001_initial.sql             # Initial database schema
├── 002_session_storage.sql     # Session storage tables and utilities
└── 003_analytics.sql           # Analytics and monitoring tables
```

## Running Migrations

### Using sqlx-cli (Recommended)

1. Install sqlx-cli if you haven't already:

   ```bash
   cargo install sqlx-cli --features postgres
   ```

2. Set your database URL:

   ```bash
   export DATABASE_URL="postgresql://username:password@localhost:5432/webauthn_db"
   ```

3. Run migrations:
   ```bash
   sqlx migrate run
   ```

### Manual Migration

If you prefer to run migrations manually:

```bash
# Run each migration file in order
psql -d webauthn_db -f migrations/001_initial.sql
psql -d webauthn_db -f migrations/002_session_storage.sql
psql -d webauthn_db -f migrations/003_analytics.sql
```

## Migration Files

### 001_initial.sql

Creates the core tables:

- `invite_codes` - Stores invitation codes for user registration
- `users` - User accounts
- `webauthn_credentials` - WebAuthn/FIDO2 credentials
- `tower_sessions` - Session storage for tower-sessions

### 002_session_storage.sql

Enhances session storage with:

- Additional indexes for performance
- Cleanup function for expired sessions
- Comprehensive documentation

### 003_analytics.sql

Adds analytics and monitoring:

- `request_analytics` - HTTP request tracking
- Performance monitoring fields
- Distributed tracing support

## Database Queries

SQL queries used by the application are defined inline in `server/src/database.rs`. This keeps the query logic close to the Rust code for better maintainability and allows for compile-time validation with sqlx.

## Best Practices

1. **Never modify existing migration files** - Always create new migrations for schema changes
2. **Test migrations** on a copy of your data before applying to production
3. **Backup your database** before running migrations
4. **Use transactions** for complex migrations to ensure atomicity
5. **Document changes** in migration files with comments

## Troubleshooting

### Migration Fails

- Check database connection settings
- Ensure PostgreSQL version compatibility
- Verify user permissions for DDL operations

### sqlx-cli Issues

- Ensure DATABASE_URL is correctly set
- Check that the database exists and is accessible
- Verify migration files are in the correct format

### Starting Fresh

If you need to completely reset your database schema:

1. Drop and recreate the database:

   ```sql
   DROP DATABASE IF EXISTS webauthn_db;
   CREATE DATABASE webauthn_db;
   ```

2. Run all migrations from the beginning:
   ```bash
   sqlx migrate run
   ```

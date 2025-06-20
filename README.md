# a wild ride with claude

this repo is mostly all ai-generated code. mostly a wild adventure staring into the llm abyss 😎

both frightening and thrilling at the same time.

i started with a browser passkey demo from [webauthn-rs](https://github.com/kanidm/webauthn-rs/tree/master/tutorial/server/axum)

💸 but then ...well i just kept lobbing prompts and claude kept pumping out code. i burned all my 10-dollar-a-month credits real quick. i'm also currently ~$28.16~ $39.84 deep in premium credit usage.

😳 if you're curious i tried to log the prompts i used (or at least the ones i could retrieve) over in [docs/prompts.md](docs/prompts.md)

⏲️ lolol, are we cooked??

everything below is what claude ai though should be in the root README:

# Axum WebAuthn Server with Invite Codes

This demonstrates using Axum as the backend for a WebAuthn authentication system with invite code functionality and comprehensive configuration management.

## Features

- **WebAuthn Authentication**: Passwordless authentication using FIDO2/WebAuthn
- **Invite Code System**: Registration requires valid invite codes
- **PostgreSQL Storage**: All data (users, credentials, invite codes, sessions) stored in PostgreSQL
- **CLI Administration**: Command-line tool for managing invite codes and configuration
- **JSONC Configuration**: Feature-rich configuration system with JSON Schema support
- **Request Analytics**: Built-in analytics and metrics collection
- **Flexible Architecture**: Easy deployment and customization

## Prerequisites

- Rust (latest stable)
- PostgreSQL database
- Modern web browser with WebAuthn support
- JSON-aware editor (VS Code, IntelliJ, etc.) for configuration editing

## Quick Start

### Using the Development Script (Recommended)

1. **Set up PostgreSQL database**

   ```bash
   # Create a PostgreSQL database
   createdb webauthn_db
   ```

2. **Initialize configuration**

   ```bash
   # Generate default configuration file
   cargo run --bin cli config init

   # Generate JSON Schema for editor support
   cargo run --bin cli config schema
   ```

3. **Configure your setup**

   ```bash
   # Edit the configuration file (supports comments!)
   # Your editor should provide autocomplete and validation
   edit config.jsonc

   # Generate .env file for Docker/SQLx compatibility
   cargo run --bin cli config generate-env

   # Set your database password
   export DATABASE_PASSWORD="your_secure_password"
   ```

4. **Run the development script**

   ```bash
   ./start_dev.sh
   ```

   This script will:

   - Validate your configuration
   - Check database connectivity
   - Run migrations automatically
   - Generate initial invite codes if none exist
   - Display available invite codes
   - Start the server

### Manual Setup

1. **Database Setup**

   ```bash
   # Create a PostgreSQL database
   createdb webauthn_db
   ```

2. **Configuration Setup**

   ```bash
   # Initialize default configuration
   cargo run --bin cli config init

   # Generate JSON Schema for your editor
   cargo run --bin cli config schema

   # Edit configuration to match your setup
   edit config.jsonc

   # Validate your configuration
   cargo run --bin cli config validate

   # Generate .env file for compatibility
   cargo run --bin cli config generate-env
   ```

3. **Set Environment Variables**

   ```bash
   # Set your database password
   export DATABASE_PASSWORD="your_secure_password"
   ```

4. **Generate Invite Codes**

   ```bash
   # Generate invite codes (uses config defaults)
   cargo run --bin cli users generate-invite

   # Override defaults
   cargo run --bin cli users generate-invite --count 5 --length 12
   ```

## Running the Server

### JavaScript Frontend (Default)

```bash
cargo run
```

The server will start on `http://localhost:8080` and serve the JavaScript frontend.

### WASM Frontend

To use the WASM frontend instead:

1. Change the features in `Cargo.toml`:

   ```toml
   [features]
   default = ["wasm"]
   ```

2. Build the WASM files:

   ```bash
   ./build_wasm.sh
   ```

3. Run the server:
   ```bash
   cargo run
   ```

## Configuration Management

### Configuration Files

The server uses a JSONC configuration file (`config.jsonc`) with full JSON Schema support for editor assistance, plus an optional secrets file for sensitive data:

```bash
# Initialize default configuration
cargo run --bin cli config init

# Initialize configuration WITH secrets file
cargo run --bin cli config init --with-secrets

# Create just the secrets file
cargo run --bin cli config init-secrets

# Generate JSON Schema for editor support
cargo run --bin cli config schema

# Validate configuration (includes secrets validation)
cargo run --bin cli config validate

# View current configuration
cargo run --bin cli config show

# Generate clean .env file for Docker/SQLx
cargo run --bin cli config generate-env
```

### Editor Setup

For the best experience, configure your editor to use the JSON Schema:

**VS Code**: Add to settings.json:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["config.jsonc"],
      "url": "./config.schema.json"
    }
  ]
}
```

**IntelliJ/WebStorm**: Preferences → JSON Schema Mappings

### Key Configuration Sections

**Main Configuration (`config.jsonc`):**

- **`app`**: Application metadata and environment
- **`database`**: Database connection and pool settings (password comes from secrets)
- **`webauthn`**: WebAuthn/FIDO2 configuration
- **`server`**: HTTP server settings
- **`sessions`**: Session management
- **`invite_codes`**: Invite code system settings
- **`logging`**: Logging and tracing
- **`analytics`**: Request analytics and metrics
- **`static_files`**: Static file serving
- **`development`**: Development-specific settings
- **`production`**: Production deployment settings
- **`features`**: Feature flags

**Secrets Configuration (`config.secrets.jsonc`):**

- **`database`**: Database password and optional URL override
- **`app`**: Session secrets and API keys
- **`external`**: Third-party service credentials

## CLI Administration

The `cli` tool provides comprehensive management:

### Configuration Commands

```bash
# Initialize configuration (basic)
cargo run --bin cli config init

# Initialize configuration with secrets file
cargo run --bin cli config init --with-secrets

# Create/update secrets file only
cargo run --bin cli config init-secrets

# Validate configuration and secrets
cargo run --bin cli config validate

# Show merged configuration
cargo run --bin cli config show

# Generate schema for editor support
cargo run --bin cli config schema

# Generate clean .env file (no comments)
cargo run --bin cli config generate-env

# Generate .env with example values
cargo run --bin cli config generate-env --with-examples
```

### Invite Code Management

```bash
# Generate invite codes (uses config defaults)
cargo run --bin cli users generate-invite

# Override defaults
cargo run --bin cli users generate-invite --count 10 --length 12

# List all invite codes
cargo run --bin cli users list-invites

# List only active codes
cargo run --bin cli users list-invites --active-only

# Show usage statistics
cargo run --bin cli users stats
```

### Analytics Commands

```bash
# Show request analytics
cargo run --bin cli analytics analytics

# Show specific time period
cargo run --bin cli analytics analytics --hours 1

# Show user activity
cargo run --bin cli analytics user-activity --user-id USER_UUID

# Clean up old data
cargo run --bin cli analytics cleanup-analytics --days 30 --execute
```

## Using the System

1. **Generate an invite code** using the CLI tool
2. **Open the web interface** at `http://localhost:8080`
3. **Register a new user**:
   - Enter a username
   - Enter the invite code
   - Click "Register"
   - Follow your browser's WebAuthn prompts
4. **Login**:
   - Enter your username
   - Click "Login"
   - Use your authenticator (fingerprint, security key, etc.)

## Database Schema

The system uses the following tables:

- `invite_codes`: Stores invite codes and their usage status
- `users`: User accounts linked to invite codes
- `webauthn_credentials`: WebAuthn credentials for each user
- `tower_sessions`: Session storage
- `request_analytics`: Request tracking and analytics data

Migrations are automatically run when the server starts (configurable via `database.migrations.auto_run`).

## Security Notes

- Invite codes are single-use and automatically deactivated after registration
- All WebAuthn credentials are stored securely in the database
- Sessions are stored server-side (configurable: memory or PostgreSQL)
- The system uses secure session cookies with appropriate flags
- Configuration validation ensures secure defaults for production
- Request analytics help monitor for suspicious activity
- Production mode enforces HTTPS and additional security headers

## Development

### Configuration vs Environment Variables

The server primarily uses `config.jsonc` + `config.secrets.jsonc` for configuration, but also supports these environment variables:

- `CONFIG_PATH`: Path to configuration file (default: `config.jsonc`)
- `DATABASE_PASSWORD` or `POSTGRES_PASSWORD`: Database password (fallback if no secrets file)
- `RUST_LOG`: Logging level (overrides config)

**Secrets Priority Order:**

1. `config.secrets.jsonc` file (preferred)
2. Environment variables (fallback)
3. Generated `.env` file (Docker/tooling compatibility)

### Build Features

- `javascript` (default): Serves JavaScript frontend
- `wasm`: Serves WASM frontend

Configure via `static_files.frontend_type` in config.jsonc.

### Database Migrations

Migrations are automatically run when the server starts (configurable via `database.migrations.auto_run`) or when using the CLI tool.

### Development Workflow

1. **Initial Setup:**

   ```bash
   # Create config and secrets together
   cargo run --bin cli config init --with-secrets

   # Edit your actual secrets (use strong passwords!)
   edit config.secrets.jsonc

   # Set proper file permissions
   chmod 600 config.secrets.jsonc
   ```

2. **Daily Development:**

   ```bash
   # Edit main configuration (with schema validation)
   edit config.jsonc

   # Validate everything
   cargo run --bin cli config validate

   # Start development server (secrets-aware)
   ./start_dev.sh
   ```

3. **Monitoring:** Monitor logs and analytics via CLI commands

## Troubleshooting

### Configuration Issues

1. **Configuration Validation Errors**

   ```bash
   # Check configuration validity
   cargo run --bin cli config validate

   # Show current configuration
   cargo run --bin cli config show
   ```

2. **Editor Schema Support**
   ```bash
   # Generate/update JSON Schema
   cargo run --bin cli config schema
   ```

### Database Issues

1. **Connection Errors**

   - Verify PostgreSQL is running
   - Check database settings in `config.jsonc`
   - Ensure `DATABASE_PASSWORD` environment variable is set
   - Test: `cargo run --bin cli users stats`

2. **Migration Issues**
   - Migrations run automatically by default
   - Disable with `database.migrations.auto_run: false`
   - Manual migration info in database module

### WebAuthn Issues

1. **RP ID/Origin Mismatch**

   - Check `webauthn.rp_id` matches your domain
   - Ensure `webauthn.rp_origin` is correct and accessible
   - Use `localhost` (not `127.0.0.1`) for development

2. **HTTPS Requirements**
   - Production WebAuthn requires HTTPS
   - Configure `server.tls` section for HTTPS
   - Set `production.require_https: true`

### Debug Information

Enable detailed logging:

```bash
# Edit config.jsonc
{
  "logging": {
    "level": "debug"
  }
}

# Or override with environment
RUST_LOG=debug cargo run
```

Check analytics for request patterns:

```bash
cargo run --bin cli analytics analytics --hours 1
```

## Development Notes

- Sessions are currently stored in memory for simplicity. In production, consider using PostgreSQL-backed sessions by uncommenting the PostgresStore code in `main.rs`
- The development script (`start_dev.sh`) provides a convenient way to set up and run the server
- Database migrations run automatically when the server starts

## Production Deployment

### Configuration for Production

1. **Create production configurations**:

   ```bash
   # Copy main config and secrets
   cp config.jsonc config.production.jsonc
   cp config.secrets.jsonc config.secrets.production.jsonc
   ```

2. **Edit production settings**:

   ```jsonc
   // config.production.jsonc
   {
     "app": {
       "environment": "production",
     },
     "server": {
       "tls": {
         "enabled": true,
         "cert_file": "/path/to/cert.pem",
         "key_file": "/path/to/key.pem",
       },
     },
     "sessions": {
       "secure": true,
       "store_type": "postgres", // Future feature
     },
     "production": {
       "require_https": true,
       "security_headers": true,
       "rate_limiting": {
         "enabled": true,
       },
     },
   }
   ```

3. **Update production secrets**:

   ```bash
   # Edit with production credentials
   edit config.secrets.production.jsonc
   chmod 600 config.secrets.production.jsonc
   ```

4. **Deploy with production config**:
   ```bash
   export CONFIG_PATH="config.production.jsonc"
   # Secrets are automatically loaded from config.secrets.production.jsonc
   ```

### Production Checklist

- ✅ HTTPS enabled (`server.tls.enabled: true`)
- ✅ Secure cookies (`sessions.secure: true`)
- ✅ Security headers (`production.security_headers: true`)
- ✅ Rate limiting (`production.rate_limiting.enabled: true`)
- ✅ Database connection pooling configured
- ✅ Log retention policy set (`analytics.retention_days`)
- ✅ Monitoring endpoints configured (`analytics.metrics.enabled`)
- ✅ Backup strategy for PostgreSQL
- ✅ Firewall configuration
- ✅ Regular security updates

### Monitoring

Enable metrics collection:

```jsonc
{
  "analytics": {
    "metrics": {
      "enabled": true,
      "prometheus_endpoint": "/metrics",
      "health_endpoint": "/health",
    },
  },
}
```

Monitor via CLI:

```bash
# Regular health checks (automatically finds secrets file)
cargo run --bin cli --config config.production.jsonc analytics analytics

# User activity monitoring
cargo run --bin cli --config config.production.jsonc analytics user-activity --user-id UUID

# Validate production setup
cargo run --bin cli --config config.production.jsonc --secrets config.secrets.production.jsonc config validate
```

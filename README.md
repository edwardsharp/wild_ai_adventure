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
   cargo run --bin webauthn-admin config init
   
   # Generate JSON Schema for editor support
   cargo run --bin webauthn-admin config schema
   ```

3. **Configure your setup**
   ```bash
   # Edit the configuration file (supports comments!)
   # Your editor should provide autocomplete and validation
   edit config.jsonc
   
   # Generate .env file for Docker/SQLx compatibility
   cargo run --bin webauthn-admin config generate-env
   
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
   cargo run --bin webauthn-admin config init
   
   # Generate JSON Schema for your editor
   cargo run --bin webauthn-admin config schema
   
   # Edit configuration to match your setup
   edit config.jsonc
   
   # Validate your configuration
   cargo run --bin webauthn-admin config validate
   
   # Generate .env file for compatibility
   cargo run --bin webauthn-admin config generate-env
   ```

3. **Set Environment Variables**
   ```bash
   # Set your database password
   export DATABASE_PASSWORD="your_secure_password"
   ```

4. **Generate Invite Codes**
   ```bash
   # Generate invite codes (uses config defaults)
   cargo run --bin webauthn-admin generate-invite
   
   # Override defaults
   cargo run --bin webauthn-admin generate-invite --count 5 --length 12
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

### Configuration File

The server uses a JSONC configuration file (`config.jsonc`) with full JSON Schema support for editor assistance:

```bash
# Initialize default configuration
cargo run --bin webauthn-admin config init

# Generate JSON Schema for editor support
cargo run --bin webauthn-admin config schema

# Validate configuration
cargo run --bin webauthn-admin config validate

# View current configuration
cargo run --bin webauthn-admin config show

# Generate .env file for Docker/SQLx
cargo run --bin webauthn-admin config generate-env
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

- **`app`**: Application metadata and environment
- **`database`**: Database connection and pool settings
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

## CLI Administration

The `webauthn-admin` CLI tool provides comprehensive management:

### Configuration Commands

```bash
# Initialize configuration
cargo run --bin webauthn-admin config init

# Validate configuration
cargo run --bin webauthn-admin config validate

# Show configuration
cargo run --bin webauthn-admin config show

# Generate schema for editor support
cargo run --bin webauthn-admin config schema

# Generate .env file
cargo run --bin webauthn-admin config generate-env
```

### Invite Code Management

```bash
# Generate invite codes (uses config defaults)
cargo run --bin webauthn-admin generate-invite

# Override defaults
cargo run --bin webauthn-admin generate-invite --count 10 --length 12

# List all invite codes
cargo run --bin webauthn-admin list-invites

# List only active codes
cargo run --bin webauthn-admin list-invites --active-only

# Show usage statistics
cargo run --bin webauthn-admin stats
```

### Analytics Commands

```bash
# Show request analytics
cargo run --bin webauthn-admin analytics

# Show specific time period
cargo run --bin webauthn-admin analytics --hours 1

# Show user activity
cargo run --bin webauthn-admin user-activity --user-id USER_UUID

# Clean up old data
cargo run --bin webauthn-admin cleanup-analytics --days 30 --execute
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

The server primarily uses `config.jsonc` for configuration, but also supports these environment variables:

- `CONFIG_PATH`: Path to configuration file (default: `config.jsonc`)
- `DATABASE_PASSWORD` or `POSTGRES_PASSWORD`: Database password
- `RUST_LOG`: Logging level (overrides config)

### Build Features

- `javascript` (default): Serves JavaScript frontend
- `wasm`: Serves WASM frontend

Configure via `static_files.frontend_type` in config.jsonc.

### Database Migrations

Migrations are automatically run when the server starts (configurable via `database.migrations.auto_run`) or when using the CLI tool.

### Development Workflow

1. Edit `config.jsonc` (with schema validation)
2. Run `cargo run --bin webauthn-admin config validate`
3. Use `./start_dev.sh` for development server
4. Monitor logs and analytics via CLI commands

## Troubleshooting

### Configuration Issues

1. **Configuration Validation Errors**
   ```bash
   # Check configuration validity
   cargo run --bin webauthn-admin config validate
   
   # Show current configuration
   cargo run --bin webauthn-admin config show
   ```

2. **Editor Schema Support**
   ```bash
   # Generate/update JSON Schema
   cargo run --bin webauthn-admin config schema
   ```

### Database Issues

1. **Connection Errors**
   - Verify PostgreSQL is running
   - Check database settings in `config.jsonc`
   - Ensure `DATABASE_PASSWORD` environment variable is set
   - Test: `cargo run --bin webauthn-admin stats`

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
cargo run --bin webauthn-admin analytics --hours 1
```

## Development Notes

- Sessions are currently stored in memory for simplicity. In production, consider using PostgreSQL-backed sessions by uncommenting the PostgresStore code in `main.rs`
- The development script (`start_dev.sh`) provides a convenient way to set up and run the server
- Database migrations run automatically when the server starts

## Production Deployment

### Configuration for Production

1. **Create production config**:
   ```bash
   cp config.jsonc config.production.jsonc
   ```

2. **Edit production settings**:
   ```jsonc
   {
     "app": {
       "environment": "production"
     },
     "server": {
       "tls": {
         "enabled": true,
         "cert_file": "/path/to/cert.pem",
         "key_file": "/path/to/key.pem"
       }
     },
     "sessions": {
       "secure": true,
       "store_type": "postgres"  // Future feature
     },
     "production": {
       "require_https": true,
       "security_headers": true,
       "rate_limiting": {
         "enabled": true
       }
     }
   }
   ```

3. **Environment variables**:
   ```bash
   export CONFIG_PATH="config.production.jsonc"
   export DATABASE_PASSWORD="secure_production_password"
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
      "health_endpoint": "/health"
    }
  }
}
```

Monitor via CLI:
```bash
# Regular health checks
cargo run --bin webauthn-admin --config config.production.jsonc analytics

# User activity monitoring
cargo run --bin webauthn-admin --config config.production.jsonc user-activity --user-id UUID
```
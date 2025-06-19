# Axum WebAuthn Server with Invite Codes

This demonstrates using Axum as the backend for a WebAuthn authentication system with invite code functionality.

## Features

- **WebAuthn Authentication**: Passwordless authentication using FIDO2/WebAuthn
- **Invite Code System**: Registration requires valid invite codes
- **PostgreSQL Storage**: All data (users, credentials, invite codes, sessions) stored in PostgreSQL
- **CLI Administration**: Command-line tool for managing invite codes

## Prerequisites

- Rust (latest stable)
- PostgreSQL database
- Modern web browser with WebAuthn support

## Quick Start

### Using the Development Script (Recommended)

1. **Set up PostgreSQL database**
   ```bash
   # Create a PostgreSQL database
   createdb webauthn_db
   ```

2. **Configure environment**
   ```bash
   # Copy and edit the environment file
   cp .env.example .env
   # Edit .env with your database credentials
   ```

3. **Run the development script**
   ```bash
   ./start_dev.sh
   ```

   This script will:
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
   
   # Set the database URL
   export DATABASE_URL="postgresql://username:password@localhost:5432/webauthn_db"
   ```

2. **Environment Configuration**
   ```bash
   # Copy the example environment file
   cp .env.example .env
   
   # Edit .env with your database credentials
   ```

3. **Install Dependencies**
   ```bash
   cargo build
   ```

4. **Generate Invite Codes**
   ```bash
   # Generate a single invite code
   cargo run --bin webauthn-admin generate-invite
   
   # Generate multiple invite codes
   cargo run --bin webauthn-admin generate-invite --count 5
   
   # Generate longer invite codes
   cargo run --bin webauthn-admin generate-invite --length 12
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

## CLI Administration

The `webauthn-admin` CLI tool provides invite code management:

### Generate Invite Codes

```bash
# Generate a single 8-character invite code
cargo run --bin webauthn-admin generate-invite

# Generate 10 invite codes
cargo run --bin webauthn-admin generate-invite --count 10

# Generate 12-character invite codes
cargo run --bin webauthn-admin generate-invite --length 12
```

### List Invite Codes

```bash
# List all invite codes
cargo run --bin webauthn-admin list-invites

# List only active/unused invite codes
cargo run --bin webauthn-admin list-invites --active-only
```

### View Statistics

```bash
# Show invite code usage statistics
cargo run --bin webauthn-admin stats
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

## Security Notes

- Invite codes are single-use and automatically deactivated after registration
- All WebAuthn credentials are stored securely in the database
- Sessions are stored server-side in PostgreSQL
- The system uses secure session cookies with appropriate flags

## Development

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required)
- `RUST_LOG`: Logging level (optional, defaults to INFO)

### Features

- `javascript` (default): Serves JavaScript frontend
- `wasm`: Serves WASM frontend

### Database Migrations

Migrations are automatically run when the server starts or when using the CLI tool.

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Verify PostgreSQL is running
   - Check `DATABASE_URL` is correct
   - Ensure database exists and user has proper permissions

2. **WebAuthn Errors**
   - Ensure you're accessing via `localhost:8080` (not `127.0.0.1`)
   - Use HTTPS in production environments
   - Check browser WebAuthn support

3. **Invite Code Issues**
   - Verify invite codes are active and unused
   - Check for typos in invite code entry
   - Generate new codes if needed

### Logs

Enable debug logging for more detailed information:

```bash
RUST_LOG=debug cargo run
```

## Development Notes

- Sessions are currently stored in memory for simplicity. In production, consider using PostgreSQL-backed sessions by uncommenting the PostgresStore code in `main.rs`
- The development script (`start_dev.sh`) provides a convenient way to set up and run the server
- Database migrations run automatically when the server starts

## Production Deployment

For production use:

1. Use HTTPS (required for WebAuthn in production)
2. Set secure session configuration
3. Use PostgreSQL-backed sessions instead of MemoryStore
4. Use connection pooling for database
5. Set up proper database backups
6. Monitor invite code usage
7. Consider rate limiting for registration attempts
8. Set appropriate CORS headers if serving a separate frontend
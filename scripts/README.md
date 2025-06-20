# Scripts Directory

This directory contains all shell scripts for the WebAuthn project. All scripts are designed to be run from the project root directory and will automatically change to the correct working directory.

## Available Scripts

### 🚀 Development Scripts

#### `start_dev.sh`
**Purpose**: Complete development environment setup and server startup

**Usage**: `./scripts/start_dev.sh`

**What it does**:
- Creates default configuration files if missing
- Validates configuration and secrets
- Generates .env file for Docker/SQLx compatibility
- Tests database connection
- Generates invite codes if none exist
- Shows configuration summary
- Starts the WebAuthn server

**Dependencies**: Requires Rust, PostgreSQL, and valid configuration

---

#### `build_wasm.sh`
**Purpose**: Build WebAssembly components

**Usage**: `./scripts/build_wasm.sh`

**What it does**:
- Builds WASM packages using wasm-pack
- Outputs to the appropriate assets directory

**Dependencies**: Requires wasm-pack

---

### 🗄️ Database Scripts

#### `run_migrations.sh`
**Purpose**: Run database migrations with multiple options

**Usage**:
```bash
./scripts/run_migrations.sh [COMMAND]
```

**Commands**:
- `run` (default) - Run all pending migrations
- `status` - Check current migration status
- `reset` - Reset database (DANGEROUS - development only)
- `help` - Show detailed usage information

**Features**:
- Supports both sqlx-cli and manual psql migrations
- Loads environment variables from .env
- Comprehensive error handling and logging
- Connection testing before migration

**Dependencies**: PostgreSQL client tools (psql), optionally sqlx-cli

---

#### `reset_db.sh`
**Purpose**: Complete database reset for development

**Usage**: `./scripts/reset_db.sh`

**What it does**:
- Stops Docker containers and removes volumes
- Clears SQLx cache (.sqlx directory)
- Starts fresh database container
- Runs migrations automatically

**Dependencies**: Docker, Docker Compose, PostgreSQL

---

### 🧪 Testing Scripts

#### `build_and_test.sh`
**Purpose**: Comprehensive build and test automation

**Usage**: `./scripts/build_and_test.sh [COMMAND]`

**Commands**: See the script's help output for full details

**Features**:
- Full CI/CD pipeline automation
- Code coverage reporting
- Integration and end-to-end testing
- Performance testing
- TypeScript client testing

---

#### `health-check.sh`
**Purpose**: Health check for monitoring and deployment

**Usage**: `./scripts/health-check.sh`

**What it does**:
- Tests API endpoints
- Validates server health
- Returns appropriate exit codes for monitoring systems

---

## Running Scripts

### From Project Root (Recommended)
```bash
# Run any script from the project root
./scripts/start_dev.sh
./scripts/run_migrations.sh status
./scripts/reset_db.sh
```

### From Scripts Directory
```bash
# Scripts will automatically change to project root
cd scripts
./start_dev.sh
./run_migrations.sh help
```

## Environment Variables

Most scripts respect these environment variables:

### Database Configuration
- `POSTGRES_HOST` - Database host (default: localhost)
- `POSTGRES_PORT` - Database port (default: 5432)
- `POSTGRES_DB` - Database name (default: webauthn_db)
- `POSTGRES_USER` - Database user (default: postgres)
- `POSTGRES_PASSWORD` - Database password (default: password)

### Alternative Database Variables
- `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASSWORD`

### Development
- `RUST_LOG` - Logging level (info, debug, trace, etc.)
- `DATABASE_URL` - Full database connection string

## Configuration Files

Scripts expect these files in the project root:
- `config.jsonc` - Main configuration
- `config.secrets.jsonc` - Secrets (optional)
- `.env` - Environment variables (auto-generated)
- `docker-compose.yml` - Docker services

## Dependencies

### Required for all scripts:
- Bash shell
- Project must be run from root directory (handled automatically)

### Script-specific dependencies:
- **Database scripts**: PostgreSQL client tools (`psql`)
- **Development scripts**: Rust toolchain (`cargo`)
- **WASM scripts**: `wasm-pack`
- **Docker scripts**: Docker and Docker Compose
- **Testing scripts**: Node.js (for TypeScript tests)

## Troubleshooting

### "Permission denied" errors
```bash
chmod +x scripts/*.sh
```

### "Command not found" errors
- Ensure required dependencies are installed
- Check PATH includes necessary tools

### Database connection issues
- Verify PostgreSQL is running
- Check configuration in `config.jsonc`
- Ensure environment variables are set correctly

### Script execution from wrong directory
All scripts automatically change to the project root directory, so this should not be an issue. If you encounter path-related errors, ensure the script is being run from within the project structure.

## Contributing

When adding new scripts:
1. Place them in this `scripts/` directory
2. Add the directory change header: `cd "$(dirname "$0")/.."`
3. Make them executable: `chmod +x scripts/your-script.sh`
4. Update this README with documentation
5. Follow the existing naming conventions and error handling patterns

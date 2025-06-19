use crate::analytics::AnalyticsService;
use crate::config::{AppConfig, ConfigError, SecretsConfig};
use crate::database::Database;
use clap::{Parser, Subcommand};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;

use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "webauthn-admin")]
#[command(about = "WebAuthn administration CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to configuration file
    #[arg(long, short, default_value = "config.jsonc")]
    pub config: PathBuf,

    /// Path to secrets configuration file
    #[arg(long, default_value = "config.secrets.jsonc")]
    pub secrets: PathBuf,

    /// Database URL (overrides config file)
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Generate a new invite code
    GenerateInvite {
        /// Number of invite codes to generate
        #[arg(short, long)]
        count: Option<u32>,
        /// Length of the invite code
        #[arg(short, long)]
        length: Option<usize>,
    },
    /// List all invite codes
    ListInvites {
        /// Show only active invite codes
        #[arg(short, long)]
        active_only: bool,
    },
    /// Show invite code statistics
    Stats,
    /// Show request analytics
    Analytics {
        /// Time period in hours
        #[arg(long, default_value = "24")]
        hours: i32,
        /// Number of top paths to show
        #[arg(long, default_value = "10")]
        limit: i64,
    },
    /// Show user request history
    UserActivity {
        /// User ID to look up
        #[arg(long)]
        user_id: String,
        /// Number of recent requests to show
        #[arg(long, default_value = "20")]
        limit: i64,
    },
    /// Clean up old analytics data
    CleanupAnalytics {
        /// Days of data to keep
        #[arg(long, default_value = "30")]
        days: i32,
        /// Actually perform the cleanup (dry run by default)
        #[arg(long)]
        execute: bool,
    },
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommands {
    /// Generate a default configuration file
    Init {
        /// Force overwrite existing config file
        #[arg(short, long)]
        force: bool,
        /// Also generate secrets file
        #[arg(long)]
        with_secrets: bool,
    },
    /// Validate the configuration file
    Validate,
    /// Generate default secrets configuration
    InitSecrets {
        /// Force overwrite existing secrets file
        #[arg(short, long)]
        force: bool,
    },
    /// Generate JSON Schema for editor support
    Schema {
        /// Output path for schema file
        #[arg(short, long, default_value = "config.schema.json")]
        output: PathBuf,
    },
    /// Generate .env file from configuration
    GenerateEnv {
        /// Output path for .env file
        #[arg(short, long, default_value = ".env")]
        output: PathBuf,
        /// Include example values with comments
        #[arg(long)]
        with_examples: bool,
    },
    /// Show current configuration
    Show {
        /// Show configuration as JSON
        #[arg(long)]
        json: bool,
        /// Show only specific section
        #[arg(long)]
        section: Option<String>,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Config { command } => {
                self.handle_config_command(command.clone()).await?;
                return Ok(());
            }
            _ => {}
        }

        // For non-config commands, we need database access
        let (config, _secrets) = self.load_config_with_secrets().await?;
        let database_url = self
            .database_url
            .clone()
            .unwrap_or_else(|| config.database_url());

        let pool = PgPool::connect(&database_url).await?;
        let db = Database::new(pool.clone());

        // Run migrations
        db.migrate().await?;

        // Create analytics service
        let analytics = AnalyticsService::new(pool);

        match self.command {
            Commands::Config { .. } => unreachable!(), // Already handled above
            Commands::GenerateInvite { count, length } => {
                let count = count.unwrap_or(config.invite_codes.default_count);
                let length = length.unwrap_or(config.invite_codes.default_length);
                self.generate_invites(&db, count, length).await?;
            }
            Commands::ListInvites { active_only } => {
                self.list_invites(&db, active_only).await?;
            }
            Commands::Stats => {
                self.show_stats(&db).await?;
            }
            Commands::Analytics { hours, limit } => {
                self.show_analytics(&analytics, hours, limit).await?;
            }
            Commands::UserActivity { ref user_id, limit } => {
                self.show_user_activity(&analytics, user_id, limit).await?;
            }
            Commands::CleanupAnalytics { days, execute } => {
                self.cleanup_analytics(&analytics, days, execute).await?;
            }
        }

        Ok(())
    }

    async fn load_config(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let (config, _) = self.load_config_with_secrets().await?;
        Ok(config)
    }

    async fn load_config_with_secrets(
        &self,
    ) -> Result<(AppConfig, Option<SecretsConfig>), Box<dyn std::error::Error>> {
        if !self.config.exists() {
            eprintln!(
                "⚠️  Configuration file '{}' not found.",
                self.config.display()
            );
            eprintln!("💡 Run 'cargo run --bin webauthn-admin config init' to create one.");
            eprintln!("🔄 Falling back to default configuration...");
            println!();
            return Ok((AppConfig::default(), None));
        }

        let secrets_path = if self.secrets.exists() {
            Some(&self.secrets)
        } else {
            None
        };

        match AppConfig::from_files(&self.config, secrets_path) {
            Ok((config, secrets)) => {
                if secrets.is_some() {
                    println!("🔐 Loaded secrets from {}", self.secrets.display());
                }
                Ok((config, secrets))
            }
            Err(e) => {
                eprintln!("❌ Failed to load configuration: {}", e);
                eprintln!("💡 Run 'cargo run --bin webauthn-admin config validate' for details.");
                std::process::exit(1);
            }
        }
    }

    async fn handle_config_command(
        &self,
        command: ConfigCommands,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            ConfigCommands::Init {
                force,
                with_secrets,
            } => {
                self.init_config(force, with_secrets).await?;
            }
            ConfigCommands::InitSecrets { force } => {
                self.init_secrets(force).await?;
            }
            ConfigCommands::Validate => {
                self.validate_config().await?;
            }
            ConfigCommands::Schema { output } => {
                self.generate_schema(output).await?;
            }
            ConfigCommands::GenerateEnv {
                output,
                with_examples,
            } => {
                self.generate_env_file(output, with_examples).await?;
            }
            ConfigCommands::Show { json, section } => {
                self.show_config(json, section).await?;
            }
        }
        Ok(())
    }

    async fn init_config(
        &self,
        force: bool,
        with_secrets: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.exists() && !force {
            eprintln!(
                "❌ Configuration file '{}' already exists.",
                self.config.display()
            );
            eprintln!("💡 Use --force to overwrite, or choose a different path with --config");
            return Ok(());
        }

        println!("🚀 Generating default configuration...");

        let config = AppConfig::default();
        config.write_to_file(&self.config)?;

        println!("✅ Created configuration file: {}", self.config.display());
        println!();
        if with_secrets {
            println!("🔐 Also generating secrets file...");
            self.init_secrets(force).await?;
        }

        println!("📋 Next steps:");
        println!("  1. Edit the configuration file to match your setup");
        if !with_secrets {
            println!("  2. Generate secrets file:");
            println!("     cargo run --bin webauthn-admin config init-secrets");
        }
        println!(
            "  {}. Generate JSON Schema for editor support:",
            if with_secrets { "2" } else { "3" }
        );
        println!("     cargo run --bin webauthn-admin config schema");
        println!(
            "  {}. Generate .env file for Docker/SQLx:",
            if with_secrets { "3" } else { "4" }
        );
        println!("     cargo run --bin webauthn-admin config generate-env");
        println!(
            "  {}. Validate your configuration:",
            if with_secrets { "4" } else { "5" }
        );
        println!("     cargo run --bin webauthn-admin config validate");

        Ok(())
    }

    async fn init_secrets(&self, force: bool) -> Result<(), Box<dyn std::error::Error>> {
        if self.secrets.exists() && !force {
            eprintln!(
                "❌ Secrets file '{}' already exists.",
                self.secrets.display()
            );
            eprintln!("💡 Use --force to overwrite, or choose a different path with --secrets");
            return Ok(());
        }

        println!("🔐 Generating default secrets file...");

        let secrets = SecretsConfig::default();
        secrets.write_to_file(&self.secrets)?;

        println!("✅ Created secrets file: {}", self.secrets.display());
        println!();
        println!("⚠️  SECURITY WARNING ⚠️");
        println!("📝 Please edit the secrets file and:");
        println!("  • Change all default passwords");
        println!("  • Use strong, unique credentials");
        println!("  • Ensure file permissions are restrictive (chmod 600)");
        println!("  • Never commit this file to version control");
        println!();
        println!("🔧 Set file permissions:");
        #[cfg(unix)]
        println!("  chmod 600 {}", self.secrets.display());
        #[cfg(windows)]
        println!("  Use Windows file permissions to restrict access");

        Ok(())
    }

    async fn validate_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Validating configuration...");

        if !self.config.exists() {
            eprintln!(
                "❌ Configuration file '{}' not found.",
                self.config.display()
            );
            eprintln!("💡 Run 'config init' to create a default configuration.");
            return Ok(());
        }

        // Check for secrets file
        let has_secrets = self.secrets.exists();
        if has_secrets {
            println!("🔐 Found secrets file: {}", self.secrets.display());
        } else {
            println!("⚠️  No secrets file found at: {}", self.secrets.display());
            println!("💡 Run 'config init-secrets' to create one");
        }

        let secrets_path = if has_secrets {
            Some(&self.secrets)
        } else {
            None
        };

        match AppConfig::from_files(&self.config, secrets_path) {
            Ok((config, secrets)) => {
                println!("✅ Configuration is valid!");
                println!();
                println!("📊 Configuration summary:");
                println!("  • App: {} v{}", config.app.name, config.app.version);
                println!("  • Environment: {}", config.app.environment);
                println!(
                    "  • Database: {}:{}/{}",
                    config.database.host, config.database.port, config.database.database
                );
                println!("  • Server: {}:{}", config.server.host, config.server.port);
                println!(
                    "  • WebAuthn: {} ({})",
                    config.webauthn.rp_id, config.webauthn.rp_name
                );
                println!("  • Frontend: {}", config.static_files.frontend_type);

                // Check for common issues
                let mut warnings = Vec::new();

                if config.app.environment == "production" {
                    if !config.production.require_https {
                        warnings.push("Production environment should require HTTPS");
                    }
                    if !config.sessions.secure {
                        warnings.push("Production environment should use secure cookies");
                    }
                }

                if config.webauthn.rp_origin.starts_with("http://")
                    && config.app.environment == "production"
                {
                    warnings.push("Production WebAuthn should use HTTPS");
                }

                // Validate secrets if present
                if let Some(ref secrets) = secrets {
                    println!();
                    println!("🔐 Secrets validation:");

                    if secrets.database.password == "change_me_secure_password" {
                        warnings.push("Database password is still using default value");
                    }

                    if let Some(ref session_secret) = secrets.app.session_secret {
                        if session_secret == "change_me_session_secret_32_chars_min" {
                            warnings.push("Session secret is still using default value");
                        } else if session_secret.len() < 32 {
                            warnings.push("Session secret should be at least 32 characters");
                        }
                    }

                    println!(
                        "  • Database password: {}",
                        if secrets.database.password == "change_me_secure_password" {
                            "❌ Default"
                        } else {
                            "✅ Custom"
                        }
                    );
                    println!(
                        "  • Session secret: {}",
                        match &secrets.app.session_secret {
                            Some(s) if s == "change_me_session_secret_32_chars_min" => "❌ Default",
                            Some(s) if s.len() < 32 => "⚠️  Too short",
                            Some(_) => "✅ Good",
                            None => "⚠️  Not set",
                        }
                    );
                } else {
                    warnings.push(
                        "No secrets file found - database password will come from environment",
                    );
                }

                if !warnings.is_empty() {
                    println!();
                    println!("⚠️  Warnings:");
                    for warning in warnings {
                        println!("  • {}", warning);
                    }
                }
            }
            Err(ConfigError::ParseError(msg)) => {
                eprintln!("❌ Configuration parsing failed:");
                eprintln!("   {}", msg);
                eprintln!();
                eprintln!("💡 Common issues:");
                eprintln!("  • Check for missing commas in JSON");
                eprintln!("  • Ensure all strings are quoted");
                eprintln!("  • Verify bracket/brace matching");
                eprintln!("  • Remove trailing commas");
            }
            Err(ConfigError::ValidationError(msg)) => {
                eprintln!("❌ Configuration validation failed:");
                eprintln!("   {}", msg);
                eprintln!();
                eprintln!("💡 Fix the issues above and run validation again.");
            }
            Err(e) => {
                eprintln!("❌ Failed to load configuration: {}", e);
            }
        }

        Ok(())
    }

    async fn generate_schema(&self, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("📝 Generating JSON Schema for editor support...");

        let schema = AppConfig::generate_schema()?;
        std::fs::write(&output, schema)?;

        println!("✅ Generated schema file: {}", output.display());
        println!();
        println!("🎯 Editor setup tips:");
        println!("  • VS Code: Install 'JSON' extension, configure in settings.json:");
        println!(
            r#"    "json.schemas": [{{"fileMatch": ["config.jsonc"], "url": "./{}"}}]"#,
            output.display()
        );
        println!("  • IntelliJ/WebStorm: Preferences → JSON Schema Mappings");
        println!("  • Vim/Neovim: Use ALE or coc-json with schema configuration");

        Ok(())
    }

    async fn generate_env_file(
        &self,
        output: PathBuf,
        with_examples: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Generating .env file...");

        let (config, secrets) = if self.config.exists() {
            let secrets_path = if self.secrets.exists() {
                Some(&self.secrets)
            } else {
                None
            };
            AppConfig::from_files(&self.config, secrets_path)?
        } else {
            println!("⚠️  Config file not found, using defaults");
            (AppConfig::default(), None)
        };

        let mut env_vars = config.to_env_vars();

        // Add secrets to env vars if available
        if let Some(ref secrets) = secrets {
            let secret_env_vars = secrets.to_env_vars();
            env_vars.extend(secret_env_vars);
        }
        let mut content = String::new();

        // Core variables - Database Configuration
        for (key, value) in &env_vars {
            if key.starts_with("DATABASE") || key.starts_with("POSTGRES") {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }

        // Application Configuration
        for (key, value) in &env_vars {
            if key.starts_with("APP_") || key == "RUST_LOG" {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }

        // Other environment variables
        for (key, value) in &env_vars {
            if !key.starts_with("DATABASE")
                && !key.starts_with("POSTGRES")
                && !key.starts_with("APP_")
                && key != "RUST_LOG"
            {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }

        if with_examples {
            content.push_str("\n");
            if secrets.is_none() {
                content.push_str("DATABASE_PASSWORD=your_secure_password_here\n");
                content.push_str("POSTGRES_PASSWORD=your_secure_password_here\n");
            }
            content.push_str("PGADMIN_DEFAULT_EMAIL=admin@example.com\n");
            content.push_str("PGADMIN_DEFAULT_PASSWORD=admin_password\n");
        }

        std::fs::write(&output, content)?;

        println!("✅ Generated .env file: {}", output.display());
        println!();
        println!("🔒 Security reminders:");
        println!("  • Add .env to your .gitignore file");
        if secrets.is_none() {
            println!("  • Set DATABASE_PASSWORD or POSTGRES_PASSWORD");
            println!("  • Consider using secrets file: config init-secrets");
        } else {
            println!("  • Secrets loaded from {}", self.secrets.display());
        }
        println!("  • Never commit environment files to version control");
        println!("  • Use strong, unique passwords in production");

        Ok(())
    }

    async fn show_config(
        &self,
        json: bool,
        section: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (config, secrets) = if self.config.exists() {
            let secrets_path = if self.secrets.exists() {
                Some(&self.secrets)
            } else {
                None
            };
            AppConfig::from_files(&self.config, secrets_path)?
        } else {
            println!("⚠️  Config file not found, showing defaults");
            (AppConfig::default(), None)
        };

        if let Some(ref _secrets) = secrets {
            println!("🔐 Secrets file loaded from: {}", self.secrets.display());
        }

        if json {
            if let Some(section) = section {
                // Show specific section
                let value = match section.as_str() {
                    "app" => serde_json::to_value(&config.app)?,
                    "database" => serde_json::to_value(&config.database)?,
                    "webauthn" => serde_json::to_value(&config.webauthn)?,
                    "server" => serde_json::to_value(&config.server)?,
                    "sessions" => serde_json::to_value(&config.sessions)?,
                    "logging" => serde_json::to_value(&config.logging)?,
                    "features" => serde_json::to_value(&config.features)?,
                    _ => {
                        eprintln!("❌ Unknown section: {}", section);
                        eprintln!("Available sections: app, database, webauthn, server, sessions, logging, features");
                        return Ok(());
                    }
                };
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
        } else {
            // Human-readable format
            println!("📋 Current Configuration");
            println!("========================");
            println!();
            println!("🏷️  Application:");
            println!("   Name: {}", config.app.name);
            println!("   Version: {}", config.app.version);
            println!("   Environment: {}", config.app.environment);
            println!();
            println!("🗄️  Database:");
            println!("   Host: {}", config.database.host);
            println!("   Port: {}", config.database.port);
            println!("   Database: {}", config.database.database);
            println!("   Username: {}", config.database.username);
            println!(
                "   Max Connections: {}",
                config.database.pool.max_connections
            );
            println!();
            println!("🔐 WebAuthn:");
            println!("   RP ID: {}", config.webauthn.rp_id);
            println!("   RP Name: {}", config.webauthn.rp_name);
            println!("   Origin: {}", config.webauthn.rp_origin);
            println!("   Timeout: {}ms", config.webauthn.timeout_ms);
            println!();
            println!("🌐 Server:");
            println!("   Host: {}", config.server.host);
            println!("   Port: {}", config.server.port);
            println!(
                "   TLS: {}",
                if config.server.tls.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            );
            println!();
            println!("🎫 Invite Codes:");
            println!("   Default Length: {}", config.invite_codes.default_length);
            println!("   Single Use: {}", config.invite_codes.single_use);
            println!("   Max Batch: {}", config.invite_codes.max_batch_size);
        }

        Ok(())
    }

    async fn generate_invites(
        &self,
        db: &Database,
        count: u32,
        length: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Generating {} invite code(s) of length {}...",
            count, length
        );
        println!();

        for i in 1..=count {
            let code = generate_invite_code(length);
            match db.create_invite_code(&code).await {
                Ok(invite_code) => {
                    println!(
                        "Generated invite code {}/{}: {}",
                        i, count, invite_code.code
                    );
                }
                Err(e) => {
                    eprintln!("Failed to generate invite code {}/{}: {}", i, count, e);
                }
            }
        }

        println!();
        println!("Done! Generated {} invite code(s).", count);
        Ok(())
    }

    async fn list_invites(
        &self,
        db: &Database,
        active_only: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let invite_codes = db.list_invite_codes().await?;

        let filtered_codes: Vec<_> = if active_only {
            invite_codes
                .into_iter()
                .filter(|code| code.is_active)
                .collect()
        } else {
            invite_codes
        };

        if filtered_codes.is_empty() {
            println!("No invite codes found.");
            return Ok(());
        }

        println!("Invite Codes:");
        println!(
            "{:<10} {:<8} {:<20} {:<20} {:<10}",
            "Code", "Active", "Created", "Used", "User ID"
        );
        println!("{}", "-".repeat(80));

        for code in filtered_codes {
            let used_at = code
                .used_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Never".to_string());

            let user_id = code
                .used_by_user_id
                .map(|id| id.to_string()[..8].to_string())
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:<10} {:<8} {:<20} {:<20} {:<10}",
                code.code,
                if code.is_active { "Yes" } else { "No" },
                code.created_at.format("%Y-%m-%d %H:%M"),
                used_at,
                user_id
            );
        }

        Ok(())
    }

    async fn show_stats(&self, db: &Database) -> Result<(), Box<dyn std::error::Error>> {
        let invite_codes = db.list_invite_codes().await?;

        let total_codes = invite_codes.len();
        let active_codes = invite_codes.iter().filter(|code| code.is_active).count();
        let used_codes = invite_codes
            .iter()
            .filter(|code| code.used_at.is_some())
            .count();

        println!("Invite Code Statistics:");
        println!("  Total codes: {}", total_codes);
        println!("  Active codes: {}", active_codes);
        println!("  Used codes: {}", used_codes);
        println!("  Unused codes: {}", total_codes - used_codes);

        Ok(())
    }

    async fn show_analytics(
        &self,
        analytics: &AnalyticsService,
        hours: i32,
        limit: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Request Analytics (Last {} hours):", hours);
        println!("{}", "=".repeat(50));

        // Get overall stats
        let stats = analytics.get_stats(hours).await?;
        println!("📊 Overall Statistics:");
        println!("  Total Requests: {}", stats.total_requests);
        println!("  Unique Users: {}", stats.unique_users);
        println!(
            "  Success Rate: {:.1}%",
            if stats.total_requests > 0 {
                (stats.success_count as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "  Average Duration: {:.1}ms",
            stats.avg_duration_ms.unwrap_or(0.0)
        );
        println!("  Error Count: {}", stats.error_count);
        println!();

        // Get top paths
        let top_paths = analytics.get_top_paths(hours, limit).await?;
        if !top_paths.is_empty() {
            println!("🔥 Top Paths:");
            println!("{:<40} {:<10} {:<15}", "Path", "Requests", "Avg Duration");
            println!("{}", "-".repeat(70));
            for path_stat in top_paths {
                println!(
                    "{:<40} {:<10} {:<15.1}ms",
                    path_stat.path,
                    path_stat.request_count,
                    path_stat.avg_duration_ms.unwrap_or(0.0)
                );
            }
        } else {
            println!("No requests found in the specified time period.");
        }

        Ok(())
    }

    async fn show_user_activity(
        &self,
        analytics: &AnalyticsService,
        user_id_str: &str,
        limit: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = Uuid::parse_str(user_id_str)?;

        println!("User Activity for {}:", user_id);
        println!("{}", "=".repeat(60));

        let requests = analytics.get_user_requests(user_id, limit).await?;

        if requests.is_empty() {
            println!("No requests found for this user.");
            return Ok(());
        }

        println!(
            "{:<20} {:<8} {:<30} {:<6} {:<10}",
            "Timestamp", "Method", "Path", "Status", "Duration"
        );
        println!("{}", "-".repeat(80));

        for req in requests {
            println!(
                "{:<20} {:<8} {:<30} {:<6} {:<10}ms",
                req.timestamp.format("%Y-%m-%d %H:%M:%S"),
                req.method,
                if req.path.len() > 28 {
                    format!("{}...", &req.path[..25])
                } else {
                    req.path.clone()
                },
                req.status_code,
                req.duration_ms.unwrap_or(0)
            );
        }

        Ok(())
    }

    async fn cleanup_analytics(
        &self,
        analytics: &AnalyticsService,
        days: i32,
        execute: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if execute {
            println!("🗑️  Cleaning up analytics data older than {} days...", days);
            let deleted_count = analytics.cleanup_old_data(days).await?;
            println!(
                "✅ Successfully deleted {} old analytics records.",
                deleted_count
            );
        } else {
            println!(
                "🔍 Dry run: Would clean up analytics data older than {} days",
                days
            );
            println!("   Use --execute to actually perform the cleanup.");
        }

        Ok(())
    }
}

fn generate_invite_code(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_code() {
        let code = generate_invite_code(8);
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_alphanumeric()));
        assert!(code.chars().all(|c| c.is_uppercase() || c.is_numeric()));
    }

    #[test]
    fn test_generate_invite_code_different_lengths() {
        for length in [4, 6, 8, 12] {
            let code = generate_invite_code(length);
            assert_eq!(code.len(), length);
        }
    }
}

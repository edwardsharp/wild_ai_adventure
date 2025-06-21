//! Simplified CLI that delegates to domain-specific modules

use clap::{Parser, Subcommand};
use sqlx::PgPool;

use server::config::{AppConfig, StorageBackend};
use server::database::DatabaseConnection;
use server::storage::AnalyticsService as StorageAnalyticsService;

use crate::analytics::AnalyticsCommands;
use crate::config::ConfigCommands;
use crate::users::UserCommands;

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "WebAuthn administration CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to configuration file
    #[arg(long, short, default_value = "assets/config/config.jsonc")]
    pub config: Option<String>,

    /// Path to secrets configuration file
    #[arg(long, default_value = "assets/config/config.secrets.jsonc")]
    pub secrets: Option<String>,

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
    /// User and invite code management
    #[command(subcommand)]
    Users(UserCommands),
    /// Analytics and data management
    #[command(subcommand)]
    Analytics(AnalyticsCommands),
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Config { command } => command.handle(self.config, self.secrets).await,
            Commands::Users(ref user_command) => {
                let (config, db) = self.setup_database().await?;
                user_command
                    .handle(
                        &db, 5,  // default count
                        32, // default length
                    )
                    .await
            }
            Commands::Analytics(ref analytics_command) => {
                let (config, db) = self.setup_database().await?;

                // Create analytics service based on storage configuration
                let analytics = match config.storage.analytics {
                    StorageBackend::Memory => StorageAnalyticsService::new_memory(),
                    StorageBackend::Postgres => {
                        StorageAnalyticsService::new_postgres(db.pool().clone())
                    }
                };

                analytics_command.handle(&analytics, &db).await
            }
        }
    }

    async fn setup_database(
        &self,
    ) -> Result<(AppConfig, DatabaseConnection), Box<dyn std::error::Error>> {
        // Load configuration
        let (config, _secrets) = self.load_config_with_secrets().await?;

        // Get database URL
        let database_url = self
            .database_url
            .clone()
            .unwrap_or_else(|| config.database_url());

        // Connect to database
        let pool = PgPool::connect(&database_url).await?;
        let db = DatabaseConnection::new(pool);

        // Run migrations
        db.migrate().await?;

        Ok((config, db))
    }

    async fn load_config_with_secrets(
        &self,
    ) -> Result<(AppConfig, Option<()>), Box<dyn std::error::Error>> {
        let config_path = self
            .config
            .as_deref()
            .unwrap_or("assets/config/config.jsonc");
        let secrets_path = self
            .secrets
            .as_deref()
            .unwrap_or("assets/config/config.secrets.jsonc");

        let secrets_path_opt = if std::path::Path::new(secrets_path).exists() {
            Some(secrets_path)
        } else {
            None
        };

        match AppConfig::from_files(config_path, secrets_path_opt) {
            Ok((config, secrets)) => {
                let secrets_loaded = if secrets.is_some() { Some(()) } else { None };
                Ok((config, secrets_loaded))
            }
            Err(e) => {
                eprintln!("Failed to load configuration: {}", e);
                eprintln!("Using default configuration...");
                Ok((AppConfig::default(), None))
            }
        }
    }
}

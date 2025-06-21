//! Config module
//!
//! This module handles all configuration-related CLI commands including:
//! - Configuration file generation and validation
//! - Secrets management
//! - Schema generation
//! - Environment file generation

use clap::Subcommand;
use server::config::AppConfig;
use std::path::PathBuf;

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
        #[arg(short, long, default_value = ".zed/config.schema.json")]
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

impl ConfigCommands {
    pub async fn handle(
        &self,
        config_path: Option<String>,
        secrets_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ConfigCommands::Init {
                force,
                with_secrets,
            } => Self::init_config(*force, *with_secrets).await,
            ConfigCommands::Validate => Self::validate_config(config_path, secrets_path).await,
            ConfigCommands::InitSecrets { force } => Self::init_secrets(*force).await,
            ConfigCommands::Schema { output } => Self::generate_schema(output).await,
            ConfigCommands::GenerateEnv {
                output,
                with_examples,
            } => Self::generate_env_file(output, *with_examples, config_path, secrets_path).await,
            ConfigCommands::Show { json, section } => {
                Self::show_config(*json, section.as_deref(), config_path, secrets_path).await
            }
        }
    }

    async fn init_config(
        force: bool,
        with_secrets: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = "assets/config/config.jsonc";
        let secrets_path = "assets/config/config.secrets.jsonc";

        // Ensure the assets/config directory exists
        if let Some(parent) = std::path::Path::new(config_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if config file already exists
        if std::path::Path::new(config_path).exists() && !force {
            return Err(format!(
                "Configuration file '{}' already exists. Use --force to overwrite.",
                config_path
            )
            .into());
        }

        // Generate default config
        let default_config = AppConfig::default();
        let config_content = serde_json::to_string_pretty(&default_config)?;

        // Write config file
        std::fs::write(config_path, config_content)?;
        println!("✓ Generated configuration file: {}", config_path);

        if with_secrets {
            Self::init_secrets(force).await?;
        }

        println!();
        println!("📝 Next steps:");
        println!("  1. Edit {} to customize your configuration", config_path);
        if with_secrets {
            println!("  2. Edit {} to set your secrets", secrets_path);
        } else {
            println!("  2. Run 'cli config init-secrets' to generate secrets file");
        }
        println!("  3. Run 'cli config validate' to check your configuration");

        Ok(())
    }

    async fn init_secrets(force: bool) -> Result<(), Box<dyn std::error::Error>> {
        let secrets_path = "assets/config/config.secrets.jsonc";

        // Ensure the assets/config directory exists
        if let Some(parent) = std::path::Path::new(secrets_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check if secrets file already exists
        if std::path::Path::new(secrets_path).exists() && !force {
            return Err(format!(
                "Secrets file '{}' already exists. Use --force to overwrite.",
                secrets_path
            )
            .into());
        }

        // Generate example secrets structure
        let secrets_content = r#"{
  // Database connection secrets
  "database": {
    "password": "your_database_password_here"
  },

  // Session encryption key (generate a secure random key)
  "session_key": "your_session_encryption_key_here",

  // Optional: External service API keys
  "external": {
    // "api_key": "your_api_key_here"
  }
}
"#;

        std::fs::write(secrets_path, secrets_content)?;
        println!("✓ Generated secrets file: {}", secrets_path);
        println!("⚠️  Remember to:");
        println!("  - Fill in your actual secrets");
        println!("  - Add {} to .gitignore", secrets_path);
        println!("  - Set appropriate file permissions (600)");

        Ok(())
    }

    async fn validate_config(
        config_path: Option<String>,
        secrets_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = config_path.unwrap_or_else(|| "assets/config/config.jsonc".to_string());
        let secrets_path =
            secrets_path.unwrap_or_else(|| "assets/config/config.secrets.jsonc".to_string());

        println!("🔍 Validating configuration...");
        println!("  Config file: {}", config_path);

        if std::path::Path::new(&secrets_path).exists() {
            println!("  Secrets file: {}", secrets_path);
        } else {
            println!(
                "  Secrets file: {} (not found, will use defaults)",
                secrets_path
            );
        }

        // Load and validate configuration
        let secrets_path_opt = if std::path::Path::new(&secrets_path).exists() {
            Some(&secrets_path)
        } else {
            None
        };

        match AppConfig::from_files(&config_path, secrets_path_opt) {
            Ok((config, secrets)) => {
                println!("✓ Configuration loaded successfully");

                if secrets.is_some() {
                    println!("✓ Secrets loaded successfully");
                } else {
                    println!("ℹ️  No secrets file loaded (using defaults)");
                }

                // Validate the configuration
                match config.validate() {
                    Ok(()) => {
                        println!("✓ Configuration validation passed");
                        println!();
                        println!("📊 Configuration summary:");
                        println!("  App name: {}", config.app.name);
                        println!("  Environment: {}", config.app.environment);
                        println!("  Server: {}:{}", config.server.host, config.server.port);
                        println!("  WebAuthn RP ID: {}", config.webauthn.rp_id);
                        println!("  Database: {}", config.database.host);
                        println!("  Features:");
                        println!("    Registration: {}", config.features.registration_enabled);
                        println!("    Analytics: {}", config.features.analytics_enabled);
                    }
                    Err(e) => {
                        println!("❌ Configuration validation failed:");
                        println!("  {}", e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to load configuration:");
                println!("  {}", e);
                return Err(Box::new(e));
            }
        }

        Ok(())
    }

    async fn generate_schema(output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating JSON schema...");

        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "AppConfig",
            "type": "object",
            "description": "WebAuthn Server Configuration"
        });
        let schema_content = serde_json::to_string_pretty(&schema)?;

        std::fs::write(output, schema_content)?;
        println!("✓ Generated JSON schema: {}", output.display());
        println!();
        println!("💡 Usage:");
        println!("  - Configure your editor to use this schema for assets/config/config.jsonc");
        println!("  - This enables autocompletion and validation in your editor");

        Ok(())
    }

    async fn generate_env_file(
        output: &PathBuf,
        with_examples: bool,
        config_path: Option<String>,
        secrets_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating .env file...");

        let config_path = config_path.unwrap_or_else(|| "assets/config/config.jsonc".to_string());
        let secrets_path =
            secrets_path.unwrap_or_else(|| "assets/config/config.secrets.jsonc".to_string());

        // Load configuration if it exists
        let config = if std::path::Path::new(&config_path).exists() {
            let secrets_path_opt = if std::path::Path::new(&secrets_path).exists() {
                Some(&secrets_path)
            } else {
                None
            };

            match AppConfig::from_files(&config_path, secrets_path_opt) {
                Ok((config, _)) => Some(config),
                Err(_) => {
                    println!("⚠️  Could not load config, using defaults");
                    None
                }
            }
        } else {
            None
        };

        let config = config.unwrap_or_else(AppConfig::default);

        // Generate environment variables (simplified for now)
        let mut content = String::new();

        content.push_str("# WebAuthn Server Environment Variables\n");
        content.push_str("# Generated from configuration\n\n");

        if with_examples {
            content.push_str("# Configuration and secrets file paths\n");
            content.push_str("# CONFIG_PATH=assets/config/config.jsonc\n");
            content.push_str("# SECRETS_PATH=assets/config/config.secrets.jsonc\n\n");
        }

        // Basic environment variables
        content.push_str(&format!("DATABASE_URL={}\n", config.database_url()));
        content.push_str(&format!("SERVER_HOST={}\n", config.server.host));
        content.push_str(&format!("SERVER_PORT={}\n", config.server.port));
        content.push_str(&format!("WEBAUTHN_RP_ID={}\n", config.webauthn.rp_id));
        content.push_str(&format!("WEBAUTHN_RP_NAME={}\n", config.webauthn.rp_name));
        content.push_str(&format!(
            "WEBAUTHN_RP_ORIGIN={}\n",
            config.webauthn.rp_origin
        ));

        std::fs::write(output, content)?;
        println!("✓ Generated .env file: {}", output.display());

        if with_examples {
            println!("💡 Uncomment and modify the variables you want to override");
        }

        Ok(())
    }

    async fn show_config(
        json: bool,
        section: Option<&str>,
        config_path: Option<String>,
        secrets_path: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = config_path.unwrap_or_else(|| "assets/config/config.jsonc".to_string());
        let secrets_path =
            secrets_path.unwrap_or_else(|| "assets/config/config.secrets.jsonc".to_string());

        let secrets_path_opt = if std::path::Path::new(&secrets_path).exists() {
            Some(&secrets_path)
        } else {
            None
        };

        let (config, _) = AppConfig::from_files(&config_path, secrets_path_opt)?;

        if json {
            if let Some(section) = section {
                // Show specific section as JSON
                let value = match section {
                    "app" => serde_json::to_value(&config.app)?,
                    "server" => serde_json::to_value(&config.server)?,
                    "database" => serde_json::to_value(&config.database)?,
                    "webauthn" => serde_json::to_value(&config.webauthn)?,
                    "features" => serde_json::to_value(&config.features)?,
                    "logging" => serde_json::to_value(&config.logging)?,
                    "sessions" => serde_json::to_value(&config.sessions)?,
                    "analytics" => serde_json::to_value(&config.analytics)?,
                    "storage" => serde_json::to_value(&config.storage)?,
                    "static_files" => serde_json::to_value(&config.static_files)?,
                    "development" => serde_json::to_value(&config.development)?,
                    _ => {
                        return Err(format!("Unknown section: {}", section).into());
                    }
                };
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                // Show entire config as JSON
                println!("{}", serde_json::to_string_pretty(&config)?);
            }
        } else {
            // Show human-readable format
            if let Some(section) = section {
                match section {
                    "app" => println!("App: {:#?}", config.app),
                    "server" => println!("Server: {:#?}", config.server),
                    "database" => println!("Database: {:#?}", config.database),
                    "webauthn" => println!("WebAuthn: {:#?}", config.webauthn),
                    "features" => println!("Features: {:#?}", config.features),
                    "logging" => println!("Logging: {:#?}", config.logging),
                    "sessions" => println!("Sessions: {:#?}", config.sessions),
                    "analytics" => println!("Analytics: {:#?}", config.analytics),
                    "storage" => println!("Storage: {:#?}", config.storage),
                    "static_files" => println!("Static Files: {:#?}", config.static_files),
                    "development" => println!("Development: {:#?}", config.development),
                    _ => {
                        return Err(format!("Unknown section: {}", section).into());
                    }
                }
            } else {
                println!("Configuration: {:#?}", config);
            }
        }

        Ok(())
    }
}

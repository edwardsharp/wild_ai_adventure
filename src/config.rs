use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(String),
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    #[error("Config validation failed: {0}")]
    ValidationError(String),
    #[error("JSON Schema generation failed: {0}")]
    SchemaError(String),
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// Application metadata and identification
    pub app: AppInfo,
    /// Database connection and pool settings
    pub database: DatabaseConfig,
    /// WebAuthn/FIDO2 authentication configuration
    pub webauthn: WebAuthnConfig,
    /// HTTP server configuration
    pub server: ServerConfig,
    /// Session management settings
    pub sessions: SessionConfig,
    /// Invite code system configuration
    pub invite_codes: InviteCodeConfig,
    /// Logging and tracing configuration
    pub logging: LoggingConfig,
    /// Analytics and metrics configuration
    pub analytics: AnalyticsConfig,
    /// Static file serving configuration
    pub static_files: StaticFilesConfig,
    /// Development-specific settings
    pub development: DevelopmentConfig,
    /// Production deployment settings
    pub production: ProductionConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppInfo {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,
    /// Application version
    #[serde(default = "default_app_version")]
    pub version: String,
    /// Environment (development, staging, production)
    #[serde(default = "default_environment")]
    pub environment: String,
    /// Optional description
    pub description: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseConfig {
    /// Database connection URL (will be generated from components or read from env)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Database host
    #[serde(default = "default_db_host")]
    pub host: String,
    /// Database port
    #[serde(default = "default_db_port")]
    pub port: u16,
    /// Database name
    #[serde(default = "default_db_name")]
    pub database: String,
    /// Database username
    #[serde(default = "default_db_user")]
    pub username: String,
    /// Database password (will be read from env var)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Connection pool settings
    pub pool: DatabasePoolConfig,
    /// Migration settings
    pub migrations: MigrationConfig,
}

/// Database connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabasePoolConfig {
    /// Maximum number of connections in the pool
    #[serde(default = "default_pool_max_connections")]
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    #[serde(default = "default_pool_min_connections")]
    pub min_connections: u32,
    /// Connection timeout in seconds
    #[serde(default = "default_pool_connect_timeout")]
    pub connect_timeout_seconds: u64,
    /// Idle timeout in seconds
    #[serde(default = "default_pool_idle_timeout")]
    pub idle_timeout_seconds: u64,
}

/// Database migration configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MigrationConfig {
    /// Automatically run migrations on startup
    #[serde(default = "default_auto_migrate")]
    pub auto_run: bool,
    /// Migration directory path
    #[serde(default = "default_migration_path")]
    pub path: String,
}

/// WebAuthn configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebAuthnConfig {
    /// Relying Party ID (your domain)
    #[serde(default = "default_rp_id")]
    pub rp_id: String,
    /// Relying Party name (display name)
    #[serde(default = "default_rp_name")]
    pub rp_name: String,
    /// Relying Party origin URL
    #[serde(default = "default_rp_origin")]
    pub rp_origin: String,
    /// Require resident key (device-bound credentials)
    #[serde(default)]
    pub require_resident_key: bool,
    /// User verification requirement
    #[serde(default = "default_user_verification")]
    pub user_verification: String,
    /// Timeout for WebAuthn operations in milliseconds
    #[serde(default = "default_webauthn_timeout")]
    pub timeout_ms: u32,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    /// Server host to bind to
    #[serde(default = "default_server_host")]
    pub host: String,
    /// Server port to bind to
    #[serde(default = "default_server_port")]
    pub port: u16,
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_seconds: u64,
    /// Maximum request size in bytes
    #[serde(default = "default_max_request_size")]
    pub max_request_size_bytes: usize,
    /// CORS configuration
    pub cors: CorsConfig,
    /// TLS configuration
    pub tls: TlsConfig,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorsConfig {
    /// Enable CORS
    #[serde(default)]
    pub enabled: bool,
    /// Allowed origins
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    #[serde(default)]
    pub allowed_headers: Vec<String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TlsConfig {
    /// Enable TLS
    #[serde(default)]
    pub enabled: bool,
    /// Certificate file path
    pub cert_file: Option<String>,
    /// Private key file path
    pub key_file: Option<String>,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionConfig {
    /// Session cookie name
    #[serde(default = "default_session_name")]
    pub name: String,
    /// Session max age in seconds
    #[serde(default = "default_session_max_age")]
    pub max_age_seconds: i64,
    /// Secure cookie flag
    #[serde(default)]
    pub secure: bool,
    /// SameSite cookie attribute
    #[serde(default = "default_session_same_site")]
    pub same_site: String,
    /// HttpOnly cookie flag
    #[serde(default = "default_session_http_only")]
    pub http_only: bool,
    /// Session store type
    #[serde(default = "default_session_store")]
    pub store_type: String,
}

/// Invite code configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InviteCodeConfig {
    /// Default length of generated invite codes
    #[serde(default = "default_invite_code_length")]
    pub default_length: usize,
    /// Default number of codes to generate
    #[serde(default = "default_invite_code_count")]
    pub default_count: u32,
    /// Maximum batch size for code generation
    #[serde(default = "default_invite_max_batch")]
    pub max_batch_size: u32,
    /// Expiry days for invite codes (0 = no expiry)
    #[serde(default)]
    pub expiry_days: u32,
    /// Single use enforcement
    #[serde(default = "default_invite_single_use")]
    pub single_use: bool,
    /// Case sensitive codes
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log format (json, pretty, compact)
    #[serde(default = "default_log_format")]
    pub format: String,
    /// Log to file
    pub file: Option<LogFileConfig>,
    /// Security event logging
    pub security: SecurityLoggingConfig,
}

/// Log file configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogFileConfig {
    /// Log file path
    pub path: String,
    /// Rotate logs
    #[serde(default = "default_log_rotate")]
    pub rotate: bool,
    /// Max file size in MB before rotation
    #[serde(default = "default_log_max_size")]
    pub max_size_mb: u64,
    /// Number of rotated files to keep
    #[serde(default = "default_log_keep_files")]
    pub keep_files: u32,
}

/// Security logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityLoggingConfig {
    /// Log authentication attempts
    #[serde(default = "default_true")]
    pub log_auth_attempts: bool,
    /// Log private content access
    #[serde(default = "default_true")]
    pub log_private_access: bool,
    /// Log failed invite code attempts
    #[serde(default = "default_true")]
    pub log_failed_invites: bool,
    /// Log all request analytics
    #[serde(default = "default_true")]
    pub log_request_analytics: bool,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyticsConfig {
    /// Enable analytics collection
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Data retention period in days
    #[serde(default = "default_analytics_retention")]
    pub retention_days: u32,
    /// Enable detailed request logging
    #[serde(default = "default_true")]
    pub detailed_logging: bool,
    /// Sample rate for analytics (0.0 to 1.0)
    #[serde(default = "default_analytics_sample_rate")]
    pub sample_rate: f64,
    /// Metrics endpoints configuration
    pub metrics: MetricsConfig,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricsConfig {
    /// Enable metrics endpoints
    #[serde(default)]
    pub enabled: bool,
    /// Prometheus metrics endpoint path
    #[serde(default = "default_prometheus_path")]
    pub prometheus_endpoint: String,
    /// Health check endpoint path
    #[serde(default = "default_health_path")]
    pub health_endpoint: String,
    /// Require authentication for metrics
    #[serde(default)]
    pub require_auth: bool,
}

/// Static file serving configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StaticFilesConfig {
    /// Public directory path
    #[serde(default = "default_public_dir")]
    pub public_directory: String,
    /// Private directory path
    #[serde(default = "default_private_dir")]
    pub private_directory: String,
    /// Frontend type (javascript, wasm)
    #[serde(default = "default_frontend_type")]
    pub frontend_type: String,
    /// JavaScript frontend directory
    #[serde(default = "default_js_dir")]
    pub javascript_directory: String,
    /// WASM frontend directory
    #[serde(default = "default_wasm_dir")]
    pub wasm_directory: String,
    /// Cache control settings
    pub cache: CacheConfig,
}

/// Static file cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Cache max age in seconds
    #[serde(default = "default_cache_max_age")]
    pub max_age_seconds: u32,
    /// ETags enabled
    #[serde(default = "default_true")]
    pub etags: bool,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DevelopmentConfig {
    /// Enable hot reloading
    #[serde(default)]
    pub hot_reload: bool,
    /// Enable debug middleware
    #[serde(default)]
    pub debug_middleware: bool,
    /// Auto-generate invite codes on startup
    #[serde(default)]
    pub auto_generate_invites: bool,
    /// Number of invite codes to auto-generate
    #[serde(default = "default_dev_invite_count")]
    pub auto_invite_count: u32,
    /// Seed test data
    #[serde(default)]
    pub seed_data: bool,
    /// Test users to create
    #[serde(default)]
    pub test_users: Vec<String>,
}

/// Production configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductionConfig {
    /// Require HTTPS
    #[serde(default)]
    pub require_https: bool,
    /// Enable security headers
    #[serde(default = "default_true")]
    pub security_headers: bool,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
    /// Security settings
    pub security: ProductionSecurityConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default)]
    pub enabled: bool,
    /// Requests per minute per IP
    #[serde(default = "default_rate_limit_rpm")]
    pub requests_per_minute: u32,
    /// Burst allowance
    #[serde(default = "default_rate_limit_burst")]
    pub burst: u32,
}

/// Production security configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductionSecurityConfig {
    /// Strict transport security max age
    #[serde(default = "default_hsts_max_age")]
    pub hsts_max_age: u32,
    /// Content security policy
    pub csp: Option<String>,
    /// X-Frame-Options header
    #[serde(default = "default_frame_options")]
    pub frame_options: String,
    /// X-Content-Type-Options header
    #[serde(default = "default_content_type_options")]
    pub content_type_options: String,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeatureFlags {
    /// Enable user registration
    #[serde(default = "default_true")]
    pub registration_enabled: bool,
    /// Require invite codes for registration
    #[serde(default = "default_true")]
    pub invite_codes_required: bool,
    /// Allow multiple credentials per user
    #[serde(default = "default_true")]
    pub multiple_credentials: bool,
    /// Enable CLI administration tools
    #[serde(default = "default_true")]
    pub admin_cli_enabled: bool,
    /// Enable analytics collection
    #[serde(default = "default_true")]
    pub analytics_enabled: bool,
    /// Enable public static file serving
    #[serde(default = "default_true")]
    pub public_static_files: bool,
    /// Enable private static file serving
    #[serde(default = "default_true")]
    pub private_static_files: bool,
}

// Default value functions
fn default_app_name() -> String {
    "WebAuthn Demo".to_string()
}
fn default_app_version() -> String {
    "1.0.0".to_string()
}
fn default_environment() -> String {
    "development".to_string()
}

fn default_db_host() -> String {
    "localhost".to_string()
}
fn default_db_port() -> u16 {
    5432
}
fn default_db_name() -> String {
    "webauthn_db".to_string()
}
fn default_db_user() -> String {
    "postgres".to_string()
}

fn default_pool_max_connections() -> u32 {
    10
}
fn default_pool_min_connections() -> u32 {
    1
}
fn default_pool_connect_timeout() -> u64 {
    30
}
fn default_pool_idle_timeout() -> u64 {
    600
}

fn default_auto_migrate() -> bool {
    true
}
fn default_migration_path() -> String {
    "migrations".to_string()
}

fn default_rp_id() -> String {
    "localhost".to_string()
}
fn default_rp_name() -> String {
    "WebAuthn Demo".to_string()
}
fn default_rp_origin() -> String {
    "http://localhost:8080".to_string()
}
fn default_user_verification() -> String {
    "preferred".to_string()
}
fn default_webauthn_timeout() -> u32 {
    60000
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}
fn default_server_port() -> u16 {
    8080
}
fn default_request_timeout() -> u64 {
    30
}
fn default_max_request_size() -> usize {
    1_048_576
}

fn default_cors_methods() -> Vec<String> {
    vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()]
}

fn default_session_name() -> String {
    "webauthnrs".to_string()
}
fn default_session_max_age() -> i64 {
    3600
}
fn default_session_same_site() -> String {
    "strict".to_string()
}
fn default_session_http_only() -> bool {
    true
}
fn default_session_store() -> String {
    "memory".to_string()
}

fn default_invite_code_length() -> usize {
    8
}
fn default_invite_code_count() -> u32 {
    1
}
fn default_invite_max_batch() -> u32 {
    100
}
fn default_invite_single_use() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "pretty".to_string()
}
fn default_log_rotate() -> bool {
    true
}
fn default_log_max_size() -> u64 {
    100
}
fn default_log_keep_files() -> u32 {
    10
}

fn default_analytics_retention() -> u32 {
    30
}
fn default_analytics_sample_rate() -> f64 {
    1.0
}
fn default_prometheus_path() -> String {
    "/metrics".to_string()
}
fn default_health_path() -> String {
    "/health".to_string()
}

fn default_public_dir() -> String {
    "assets/public".to_string()
}
fn default_private_dir() -> String {
    "assets/private".to_string()
}
fn default_frontend_type() -> String {
    "javascript".to_string()
}
fn default_js_dir() -> String {
    "assets/js".to_string()
}
fn default_wasm_dir() -> String {
    "assets/wasm".to_string()
}

fn default_cache_max_age() -> u32 {
    3600
}

fn default_dev_invite_count() -> u32 {
    3
}

fn default_rate_limit_rpm() -> u32 {
    60
}
fn default_rate_limit_burst() -> u32 {
    10
}

fn default_hsts_max_age() -> u32 {
    31536000
}
fn default_frame_options() -> String {
    "DENY".to_string()
}
fn default_content_type_options() -> String {
    "nosniff".to_string()
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    /// Load configuration from a JSONC file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = json5::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("JSON5 parse error: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// Generate a default configuration file
    pub fn generate_default() -> Self {
        Self {
            app: AppInfo {
                name: default_app_name(),
                version: default_app_version(),
                environment: default_environment(),
                description: Some("WebAuthn authentication server with invite codes".to_string()),
            },
            database: DatabaseConfig {
                url: None,
                host: default_db_host(),
                port: default_db_port(),
                database: default_db_name(),
                username: default_db_user(),
                password: None,
                pool: DatabasePoolConfig {
                    max_connections: default_pool_max_connections(),
                    min_connections: default_pool_min_connections(),
                    connect_timeout_seconds: default_pool_connect_timeout(),
                    idle_timeout_seconds: default_pool_idle_timeout(),
                },
                migrations: MigrationConfig {
                    auto_run: default_auto_migrate(),
                    path: default_migration_path(),
                },
            },
            webauthn: WebAuthnConfig {
                rp_id: default_rp_id(),
                rp_name: default_rp_name(),
                rp_origin: default_rp_origin(),
                require_resident_key: false,
                user_verification: default_user_verification(),
                timeout_ms: default_webauthn_timeout(),
            },
            server: ServerConfig {
                host: default_server_host(),
                port: default_server_port(),
                request_timeout_seconds: default_request_timeout(),
                max_request_size_bytes: default_max_request_size(),
                cors: CorsConfig {
                    enabled: false,
                    allowed_origins: vec!["http://localhost:8080".to_string()],
                    allowed_methods: default_cors_methods(),
                    allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                },
                tls: TlsConfig {
                    enabled: false,
                    cert_file: None,
                    key_file: None,
                },
            },
            sessions: SessionConfig {
                name: default_session_name(),
                max_age_seconds: default_session_max_age(),
                secure: false,
                same_site: default_session_same_site(),
                http_only: default_session_http_only(),
                store_type: default_session_store(),
            },
            invite_codes: InviteCodeConfig {
                default_length: default_invite_code_length(),
                default_count: default_invite_code_count(),
                max_batch_size: default_invite_max_batch(),
                expiry_days: 0,
                single_use: default_invite_single_use(),
                case_sensitive: false,
            },
            logging: LoggingConfig {
                level: default_log_level(),
                format: default_log_format(),
                file: None,
                security: SecurityLoggingConfig {
                    log_auth_attempts: true,
                    log_private_access: true,
                    log_failed_invites: true,
                    log_request_analytics: true,
                },
            },
            analytics: AnalyticsConfig {
                enabled: true,
                retention_days: default_analytics_retention(),
                detailed_logging: true,
                sample_rate: default_analytics_sample_rate(),
                metrics: MetricsConfig {
                    enabled: false,
                    prometheus_endpoint: default_prometheus_path(),
                    health_endpoint: default_health_path(),
                    require_auth: false,
                },
            },
            static_files: StaticFilesConfig {
                public_directory: default_public_dir(),
                private_directory: default_private_dir(),
                frontend_type: default_frontend_type(),
                javascript_directory: default_js_dir(),
                wasm_directory: default_wasm_dir(),
                cache: CacheConfig {
                    enabled: true,
                    max_age_seconds: default_cache_max_age(),
                    etags: true,
                },
            },
            development: DevelopmentConfig {
                hot_reload: false,
                debug_middleware: true,
                auto_generate_invites: true,
                auto_invite_count: default_dev_invite_count(),
                seed_data: false,
                test_users: vec![],
            },
            production: ProductionConfig {
                require_https: true,
                security_headers: true,
                rate_limiting: RateLimitConfig {
                    enabled: false,
                    requests_per_minute: default_rate_limit_rpm(),
                    burst: default_rate_limit_burst(),
                },
                security: ProductionSecurityConfig {
                    hsts_max_age: default_hsts_max_age(),
                    csp: None,
                    frame_options: default_frame_options(),
                    content_type_options: default_content_type_options(),
                },
            },
            features: FeatureFlags {
                registration_enabled: true,
                invite_codes_required: true,
                multiple_credentials: true,
                admin_cli_enabled: true,
                analytics_enabled: true,
                public_static_files: true,
                private_static_files: true,
            },
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // Validate server configuration
        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }

        // Validate WebAuthn configuration
        if self.webauthn.rp_id.is_empty() {
            errors.push("WebAuthn RP ID cannot be empty".to_string());
        }

        if !self.webauthn.rp_origin.starts_with("http://")
            && !self.webauthn.rp_origin.starts_with("https://")
        {
            errors.push("WebAuthn RP origin must be a valid HTTP/HTTPS URL".to_string());
        }

        // Validate user verification setting
        if !["required", "preferred", "discouraged"]
            .contains(&self.webauthn.user_verification.as_str())
        {
            errors.push(
                "WebAuthn user_verification must be 'required', 'preferred', or 'discouraged'"
                    .to_string(),
            );
        }

        // Validate database configuration
        if self.database.host.is_empty() {
            errors.push("Database host cannot be empty".to_string());
        }

        if self.database.database.is_empty() {
            errors.push("Database name cannot be empty".to_string());
        }

        if self.database.pool.max_connections == 0 {
            errors.push("Database max_connections cannot be 0".to_string());
        }

        if self.database.pool.min_connections > self.database.pool.max_connections {
            errors.push(
                "Database min_connections cannot be greater than max_connections".to_string(),
            );
        }

        // Validate logging level
        if !["trace", "debug", "info", "warn", "error"].contains(&self.logging.level.as_str()) {
            errors
                .push("Logging level must be one of: trace, debug, info, warn, error".to_string());
        }

        // Validate session same_site
        if !["strict", "lax", "none"].contains(&self.sessions.same_site.as_str()) {
            errors.push("Session same_site must be 'strict', 'lax', or 'none'".to_string());
        }

        // Validate invite code configuration
        if self.invite_codes.default_length < 4 {
            errors.push("Invite code length must be at least 4 characters".to_string());
        }

        if self.invite_codes.max_batch_size == 0 {
            errors.push("Invite code max_batch_size cannot be 0".to_string());
        }

        // Validate analytics
        if self.analytics.sample_rate < 0.0 || self.analytics.sample_rate > 1.0 {
            errors.push("Analytics sample_rate must be between 0.0 and 1.0".to_string());
        }

        // Validate frontend type
        if !["javascript", "wasm"].contains(&self.static_files.frontend_type.as_str()) {
            errors.push("Frontend type must be 'javascript' or 'wasm'".to_string());
        }

        // Production-specific validations
        if self.app.environment == "production" {
            if !self.production.require_https && self.webauthn.rp_origin.starts_with("http://") {
                errors.push("Production environment should use HTTPS for WebAuthn".to_string());
            }

            if !self.sessions.secure && self.production.require_https {
                errors.push("Production environment should use secure session cookies".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(ConfigError::ValidationError(errors.join("; ")));
        }

        Ok(())
    }

    /// Get the complete database URL, building it from components if needed
    pub fn database_url(&self) -> String {
        if let Some(url) = &self.database.url {
            url.clone()
        } else {
            // Build URL from components, password will come from environment
            let password = std::env::var("DATABASE_PASSWORD")
                .or_else(|_| std::env::var("POSTGRES_PASSWORD"))
                .unwrap_or_else(|_| "".to_string());

            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.database.username,
                password,
                self.database.host,
                self.database.port,
                self.database.database
            )
        }
    }

    /// Generate environment variables needed for Docker/SQLx
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        // Database URL for SQLx
        env_vars.insert("DATABASE_URL".to_string(), self.database_url());

        // Individual database components for Docker Compose
        env_vars.insert("POSTGRES_HOST".to_string(), self.database.host.clone());
        env_vars.insert("POSTGRES_PORT".to_string(), self.database.port.to_string());
        env_vars.insert("POSTGRES_USER".to_string(), self.database.username.clone());
        env_vars.insert("POSTGRES_DB".to_string(), self.database.database.clone());

        // Logging
        env_vars.insert("RUST_LOG".to_string(), self.logging.level.clone());

        // App info
        env_vars.insert("APP_NAME".to_string(), self.app.name.clone());
        env_vars.insert("APP_VERSION".to_string(), self.app.version.clone());
        env_vars.insert("APP_ENVIRONMENT".to_string(), self.app.environment.clone());

        env_vars
    }

    /// Generate JSON Schema for IDE support
    pub fn generate_schema() -> Result<String, ConfigError> {
        let schema = schemars::schema_for!(AppConfig);
        serde_json::to_string_pretty(&schema)
            .map_err(|e| ConfigError::SchemaError(format!("JSON serialization error: {}", e)))
    }

    /// Write a pretty-printed JSONC configuration to a file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = self.to_jsonc_string()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert to a JSONC string with helpful comments
    pub fn to_jsonc_string(&self) -> Result<String, ConfigError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(format!("JSON serialization error: {}", e)))?;

        // Add header comment
        let header = r#"// WebAuthn Server Configuration
//
// This file configures all aspects of the WebAuthn authentication server.
// For JSON Schema support in your editor, save this as config.jsonc and
// configure your editor to use the generated schema file.
//
// To generate environment variables for Docker/SQLx:
//   cargo run --bin webauthn-admin config generate-env
//
// To validate this configuration:
//   cargo run --bin webauthn-admin config validate
//
"#;

        Ok(format!("{}{}", header, json))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::generate_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_url_generation() {
        let config = AppConfig::default();
        let url = config.database_url();
        assert!(url.starts_with("postgresql://"));
        assert!(url.contains("localhost:5432"));
    }

    #[test]
    fn test_env_vars_generation() {
        let config = AppConfig::default();
        let env_vars = config.to_env_vars();
        assert!(env_vars.contains_key("DATABASE_URL"));
        assert!(env_vars.contains_key("RUST_LOG"));
        assert_eq!(env_vars.get("RUST_LOG"), Some(&"info".to_string()));
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        config.webauthn.rp_id = "".to_string();

        let result = config.validate();
        assert!(result.is_err());

        if let Err(ConfigError::ValidationError(msg)) = result {
            assert!(msg.contains("port cannot be 0"));
            assert!(msg.contains("RP ID cannot be empty"));
        }
    }

    #[test]
    fn test_json_schema_generation() {
        let schema_result = AppConfig::generate_schema();
        assert!(schema_result.is_ok());

        let schema = schema_result.unwrap();
        assert!(schema.contains("AppConfig"));
        assert!(schema.contains("properties"));
    }
}

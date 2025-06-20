use crate::config::{AppConfig, StorageBackend};
use crate::database::DatabaseConnection;
use crate::storage::{AnalyticsService, SessionStore};

use std::sync::Arc;
use webauthn_rs::prelude::*;

/*
 * Webauthn RS server side app state and setup code.
 */

// Configure the Webauthn instance by using the WebauthnBuilder. This defines
// the options needed for your site, and has some implications. One of these is that
// you can NOT change your rp_id (relying party id), without invalidating all
// webauthn credentials. Remember, rp_id is derived from your URL origin, meaning
// that it is your effective domain name.

#[derive(Clone)]
pub struct AppState {
    // Webauthn has no mutable inner state, so Arc and read only is sufficient.
    // Alternately, you could use a reference here provided you can work out
    // lifetimes.
    pub webauthn: Arc<Webauthn>,
    // Database connection for persistent storage
    pub database: DatabaseConnection,
    // Analytics service for request tracking
    pub analytics: AnalyticsService,
    // Session store for tower-sessions
    pub session_store: SessionStore,
    // Application configuration
    #[allow(dead_code)]
    pub config: AppConfig,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Build WebAuthn configuration from config
        let rp_origin = Url::parse(&config.webauthn.rp_origin)?;
        let builder = WebauthnBuilder::new(&config.webauthn.rp_id, &rp_origin)?;

        // Configure WebAuthn with settings from config
        let builder = builder.rp_name(&config.webauthn.rp_name);

        // Note: User verification and timeout settings may need to be configured
        // differently based on the webauthn-rs version and available methods
        // For now, using the basic builder configuration

        // Consume the builder and create our webauthn instance.
        let webauthn = Arc::new(builder.build()?);

        // Connect to the database using config
        let database_url = config.database_url();

        // Configure connection pool
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.database.pool.max_connections)
            .min_connections(config.database.pool.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.pool.connect_timeout_seconds,
            ))
            .idle_timeout(std::time::Duration::from_secs(
                config.database.pool.idle_timeout_seconds,
            ))
            .connect(&database_url)
            .await?;

        let database = DatabaseConnection::new(pool.clone());

        // Create analytics service based on storage configuration
        let analytics = match config.storage.analytics {
            StorageBackend::Memory => AnalyticsService::new_memory(),
            StorageBackend::Postgres => AnalyticsService::new_postgres(pool.clone()),
        };

        // Create session store based on storage configuration
        let session_store = match config.storage.sessions {
            StorageBackend::Memory => SessionStore::new_memory(),
            StorageBackend::Postgres => SessionStore::new_postgres(pool.clone()).await?,
        };

        // Run migrations if enabled
        if config.database.migrations.auto_run {
            database.migrate().await?;
        }

        Ok(AppState {
            webauthn,
            database,
            analytics,
            session_store,
            config,
        })
    }
}

use crate::analytics::AnalyticsService;
use crate::database::Database;
use sqlx::PgPool;
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
    pub database: Database,
    // Analytics service for request tracking
    pub analytics: AnalyticsService,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Effective domain name.
        let rp_id = "localhost";
        // Url containing the effective domain name
        // MUST include the port number!
        let rp_origin = Url::parse("http://localhost:8080").expect("Invalid URL");
        let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");

        // Now, with the builder you can define other options.
        // Set a "nice" relying party name. Has no security properties and
        // may be changed in the future.
        let builder = builder.rp_name("Axum Webauthn-rs");

        // Consume the builder and create our webauthn instance.
        let webauthn = Arc::new(builder.build().expect("Invalid configuration"));

        // Connect to the database
        let pool = PgPool::connect(database_url).await?;
        let database = Database::new(pool.clone());

        // Create analytics service with the same pool
        let analytics = AnalyticsService::new(pool);

        // Run migrations
        database.migrate().await?;

        Ok(AppState {
            webauthn,
            database,
            analytics,
        })
    }
}

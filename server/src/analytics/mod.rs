//! Analytics module
//!
//! This module handles all analytics and monitoring functionality including:
//! - HTTP request tracking
//! - Performance monitoring
//! - User behavior analytics
//! - Request metrics and reporting
//! - Time-series data collection

pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

// Re-export commonly used types
pub use handlers::{get_metrics, get_prometheus_metrics};
pub use middleware::{analytics_middleware, security_logging};
pub use models::{
    AnalyticsConfig, AnalyticsError, PathMetric, RequestAnalytics, RequestMetrics, TimeSeriesPoint,
};
pub use repository::AnalyticsRepository;
pub use routes::build_analytics_routes;
pub use service::{AnalyticsService, RequestAnalyticsBuilder};

// Future exports (to be added as we move more code)
// pub mod collector;
//
// pub use collector::RequestCollector;

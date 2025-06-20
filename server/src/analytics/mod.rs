//! Analytics module
//!
//! This module handles all analytics and monitoring functionality including:
//! - HTTP request tracking
//! - Performance monitoring
//! - User behavior analytics
//! - Request metrics and reporting
//! - Time-series data collection

pub mod models;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use models::{
    AnalyticsConfig, AnalyticsError, PathMetric, RequestAnalytics, RequestMetrics, TimeSeriesPoint,
};
pub use repository::AnalyticsRepository;
pub use service::{AnalyticsService, RequestAnalyticsBuilder};

// Future exports (to be added as we move more code)
// pub mod middleware;
// pub mod collector;
//
// pub use middleware::AnalyticsMiddleware;
// pub use collector::RequestCollector;

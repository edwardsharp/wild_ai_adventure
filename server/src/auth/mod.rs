//! Authentication module
//!
//! This module handles all authentication-related functionality including:
//! - User registration and management
//! - Invite code system
//! - WebAuthn/FIDO2 authentication
//! - Session management
//! - Authentication middleware

pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use models::{AuthError, InviteCode, User, UserRole, WebauthnCredential};
pub use repository::AuthRepository;

// Re-export handlers
pub use handlers::*;

// Re-export middleware
pub use middleware::{
    require_admin, require_analytics_access, require_authentication, AuthenticatedUser,
};

// Future exports (to be added as we move more code)
// pub mod service;
//
// pub use service::AuthService;

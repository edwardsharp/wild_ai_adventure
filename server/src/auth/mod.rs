//! Authentication module
//!
//! This module handles all authentication-related functionality including:
//! - User registration and management
//! - Invite code system
//! - WebAuthn/FIDO2 authentication
//! - Session management
//! - Authentication middleware

pub mod handlers;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use models::{AuthError, AuthenticatedUser, InviteCode, User, WebauthnCredential};
pub use repository::AuthRepository;

// Re-export handlers
pub use handlers::*;

// Future exports (to be added as we move more code)
// pub mod service;
// pub mod middleware;
//
// pub use service::AuthService;
// pub use middleware::AuthMiddleware;

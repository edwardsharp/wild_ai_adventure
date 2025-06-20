# Role-Based Access Control System

This document describes the role-based access control (RBAC) system implemented in the WebAuthn authentication server.

## User Roles System

- **`UserRole::Admin`** - Full system access including analytics and management
- **`UserRole::Member`** - Regular authenticated users
- Database constraint ensures only valid roles: `'admin'` or `'member'`
- First registered user automatically becomes admin

## Database Schema Updates

- Added `role` column to users table with default 'member'
- Database constraint for role validation
- All user queries now include role information

## Repository Layer Enhancements

```rust
// Create users with specific roles
auth_repo.create_user_with_role(username, invite_code, UserRole::Admin).await

// Update existing user roles
auth_repo.update_user_role(user_id, UserRole::Admin).await

// List all users (admin function)
auth_repo.list_users().await
```

## Authentication Middleware

- **`require_authentication`** - Validates session, loads user from DB
- **`require_admin`** - Admin-only routes
- **`require_analytics_access`** - Analytics access control
- **`AuthenticatedUser`** extension provides user context to handlers

## CLI Management Commands

```bash
# Create an admin user
cargo run --bin webauthn-admin -- create-admin admin_user

# List all users with roles
cargo run --bin webauthn-admin -- list-users

# Update a user's role
cargo run --bin webauthn-admin -- update-user-role someuser admin
```

## Route Protection

```rust
// Admin-only routes
let admin_routes = Router::new()
    .route("/api/analytics", get(analytics_handler))
    .route("/api/admin/metrics", get(admin_metrics))
    .layer(axum_middleware::from_fn(require_admin))
    .layer(axum_middleware::from_fn(require_authentication));

// Member routes (any authenticated user)
let protected_routes = Router::new()
    .route("/private/*path", serve_private_files)
    .route("/api/user/profile", get(user_profile))
    .layer(axum_middleware::from_fn(require_authentication));
```

## Permission Methods

```rust
user.is_admin()                    // Check admin status
user.can_access_analytics()        // Check analytics permission
user.can_manage_invites()          // Check invite management permission
```

## How It Works

1. **Registration**: First user becomes admin, subsequent users are members
2. **Authentication**: Middleware loads user from DB and adds to request context
3. **Authorization**: Route-specific middleware checks roles before allowing access
4. **Management**: CLI commands let you promote users to admin or list all users

## Usage Examples

### Creating an Admin User

The first user to register will automatically become an admin. You can also create admin users via CLI:

```bash
# Create admin with specific invite code
cargo run --bin webauthn-admin -- create-admin admin_user --invite-code ABC123

# Create admin without invite code (if configured to allow)
cargo run --bin webauthn-admin -- create-admin admin_user
```

### Managing User Roles

```bash
# List all users to see their current roles
cargo run --bin webauthn-admin -- list-users

# Promote a user to admin
cargo run --bin webauthn-admin -- update-user-role alice admin

# Demote an admin to member
cargo run --bin webauthn-admin -- update-user-role bob member
```

### Using Roles in Handlers

```rust
use axum::extract::Extension;
use crate::auth::AuthenticatedUser;

pub async fn admin_only_handler(
    Extension(user): Extension<AuthenticatedUser>
) -> Result<impl IntoResponse, StatusCode> {
    // This handler is protected by require_admin middleware
    // so we know the user is definitely an admin

    println!("Admin {} is accessing this endpoint", user.user().username);
    Ok(Json(serde_json::json!({"message": "Admin access granted"})))
}

pub async fn user_handler(
    Extension(user): Extension<AuthenticatedUser>
) -> Result<impl IntoResponse, StatusCode> {
    // This handler is protected by require_authentication
    // so user could be either admin or member

    if user.is_admin() {
        // Admin-specific logic
    } else {
        // Regular member logic
    }

    Ok(Json(serde_json::json!({
        "user": user.user().username,
        "role": user.user().role
    })))
}
```

## Security Considerations

- **Role Persistence**: User roles are stored in the database and loaded fresh on each request
- **Session Security**: Invalid sessions (user deleted from DB) are automatically cleared
- **Principle of Least Privilege**: Members get minimal access, admins get everything
- **Audit Trail**: All role changes and access attempts are logged
- **First User Protection**: The first registered user becomes admin to bootstrap the system

## Future Enhancements

The role system is designed to be extensible. Future additions could include:

- Additional roles (e.g., `Moderator`, `ReadOnly`)
- Permission-based access control beyond roles
- Role-based invite code restrictions
- Time-limited role assignments
- Role inheritance or hierarchies

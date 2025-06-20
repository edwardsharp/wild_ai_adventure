WebAuthn Demo - Public Content
===============================

This is a public text file that can be accessed by anyone without authentication.

📁 File Location: /public/readme.txt
🔓 Access Level: Public (no authentication required)
📅 Created: 2024

About This System:
-----------------
This WebAuthn demonstration showcases a complete authentication system with:

1. Invite Code Management
   - Admin CLI tool for generating invite codes
   - Single-use codes for registration control
   - Database tracking of code usage

2. WebAuthn Authentication
   - Passwordless authentication using FIDO2/WebAuthn
   - Support for hardware security keys, biometrics, etc.
   - Secure credential storage in PostgreSQL

3. Static File Serving
   - Public files (like this one) - accessible to everyone
   - Private files - only accessible to authenticated users
   - Proper middleware-based access control

4. Database Integration
   - PostgreSQL for persistent storage
   - User management with invite code tracking
   - WebAuthn credential storage
   - Session management

Technical Stack:
---------------
- Backend: Rust + Axum web framework
- Database: PostgreSQL with SQLx
- Authentication: webauthn-rs library
- Sessions: tower-sessions with memory store
- CLI: clap for administration tools

File Structure:
--------------
/public/     - Files accessible to everyone
/private/    - Files requiring authentication
/assets/js/  - Main application frontend (JavaScript)
/assets/wasm/- Alternative WASM frontend

To explore the system:
1. Visit the main page to register/login
2. Browse public content (like this file)
3. Try accessing private content (requires authentication)

For developers:
--------------
- Generate invite codes: cargo run --bin cli users generate-invite
- List codes: cargo run --bin cli users list-invites
- Start server: ./start_dev.sh or cargo run
- View logs with: RUST_LOG=debug cargo run

Security Notes:
--------------
- All WebAuthn credentials are stored securely
- Invite codes prevent unauthorized registration
- Private content requires valid authentication
- Sessions are managed server-side

This file serves as an example of public static content that's freely accessible
without any authentication requirements.

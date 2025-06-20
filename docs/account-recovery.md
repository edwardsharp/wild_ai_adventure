# Account Recovery Guide

Quick reference for managing account recovery in the WebAuthn system.

## Overview

The account recovery system allows admins to generate temporary codes that let users register new passkeys on existing accounts. This is essential when users lose devices or need to add passkeys to new devices.

## Key Concepts

- **Recovery codes are admin-generated** - users cannot create their own
- **Codes are user-specific** - each code works only for one user account
- **Single-use and time-limited** - codes expire and can only be used once
- **Links new credentials** - doesn't create new accounts, adds to existing ones

## Quick Commands

### Generate Recovery Code

```bash
# Basic recovery code (24h expiry, 12 chars)
cargo run --bin cli users generate-recovery --username alice

# Custom expiry and length
cargo run --bin cli users generate-recovery --username bob --expires-hours 12 --length 16
```

### Check User Status

```bash
# List all users
cargo run --bin cli users list-users

# Show system stats
cargo run --bin cli users stats

# List active invite/recovery codes
cargo run --bin cli users list-invites --active-only
```

## Recovery Process Workflow

### For Admins

1. **User requests help** - "I lost my phone with my passkey"
2. **Verify user identity** - confirm through alternative means
3. **Generate recovery code**:
   ```bash
   cargo run --bin cli users generate-recovery --username alice
   ```
4. **Share code securely** - email, SMS, or in-person
5. **Confirm successful recovery** - user should test new passkey

### For Users

1. **Contact admin** for account recovery
2. **Receive recovery code** through secure channel
3. **Go to registration page** (same as new user signup)
4. **Enter original username** (important: not a new username)
5. **Use recovery code** in the invite code field
6. **Register new passkey** - will be linked to existing account
7. **Test access** - login with new passkey

## Security Considerations

### Admin Best Practices

- **Verify identity** before generating recovery codes
- **Use secure channels** to share codes (avoid plain text messaging)
- **Monitor code usage** - check that codes are used by intended users
- **Set appropriate expiry** - shorter for high-security environments
- **Document recovery events** - maintain audit trail

### Code Properties

- **Default expiry**: 24 hours (configurable)
- **Default length**: 12 characters (configurable)
- **Character set**: Alphanumeric, uppercase
- **Uniqueness**: Globally unique across all codes
- **Validation**: Same security model as invite codes

## Common Scenarios

### Lost Device

**Scenario**: User's phone with passkey was lost/stolen

**Solution**:
```bash
cargo run --bin cli users generate-recovery --username alice --expires-hours 6
```
- Short expiry for security
- User registers new passkey on replacement device
- Consider revoking old passkey if device is compromised

### Device Upgrade

**Scenario**: User got new laptop and wants to migrate passkey

**Solution**:
```bash
cargo run --bin cli users generate-recovery --username bob --expires-hours 48
```
- Longer expiry for convenience
- User can take time to set up new device
- Old passkey remains valid unless explicitly removed

### Multiple Devices

**Scenario**: User wants passkey on both phone and laptop

**Solution**:
```bash
cargo run --bin cli users generate-recovery --username carol
```
- Standard expiry
- Results in multiple valid passkeys for same account
- User can authenticate from either device

### Emergency Access

**Scenario**: Critical user needs immediate access, passkey not working

**Solution**:
```bash
cargo run --bin cli users generate-recovery --username admin --expires-hours 2 --length 16
```
- Very short expiry for security
- Longer code for additional entropy
- Immediate resolution of access issue

## Database Schema

Recovery codes extend the existing `invite_codes` table:

```sql
-- New columns added to invite_codes
code_type VARCHAR(20) DEFAULT 'invite'  -- 'invite' or 'recovery'
recovery_for_user_id UUID               -- Target user for recovery
recovery_expires_at TIMESTAMPTZ         -- Recovery-specific expiry
```

## Troubleshooting

### Code Generation Issues

**Problem**: `cargo run --bin cli users generate-recovery --username alice` fails

**Solutions**:
- Check username exists: `cargo run --bin cli users list-users`
- Verify database connection: `cargo run --bin cli users stats`
- Ensure admin permissions
- Check database has recovery schema (run migrations)

### Code Not Working During Registration

**Problem**: User enters recovery code but registration fails

**Solutions**:
- Verify exact username (case-sensitive)
- Check code hasn't expired
- Confirm code hasn't been used already
- Ensure user is on registration page, not login page

### Multiple Passkeys Management

**Problem**: User has too many passkeys, wants to remove old ones

**Current limitation**: No CLI command for passkey removal yet

**Workaround**: Manual database operation
```sql
-- List user's passkeys
SELECT id, credential_id, created_at, last_used_at
FROM webauthn_credentials
WHERE user_id = 'user-uuid-here';

-- Remove specific passkey (careful!)
DELETE FROM webauthn_credentials WHERE id = 'credential-uuid-here';
```

## Integration with Existing Systems

### Invite Code Compatibility

Recovery codes use the same infrastructure as invite codes:
- Same validation logic
- Same expiry mechanisms
- Same single-use enforcement
- Same database table with type differentiation

### User Management

Recovery integrates seamlessly with existing user commands:
- `list-users` shows all users eligible for recovery
- `stats` includes recovery code counts
- `list-invites` shows both invite and recovery codes

### Configuration

Recovery codes respect existing configuration:
- Database connection settings
- Logging configuration
- Security settings
- Migration settings

## Future Enhancements

Potential improvements to consider:

1. **Self-service recovery** with additional verification
2. **Backup codes** generated during initial registration
3. **Email-based recovery** with automated code delivery
4. **Passkey management UI** for users to remove old credentials
5. **Recovery audit logs** for compliance requirements
6. **Batch recovery operations** for mass device replacements

## Reference Links

- [Main README](../README.md) - Full system documentation
- [User Management](../cli/src/users/mod.rs) - CLI implementation
- [Database Schema](../migrations/) - Migration files
- [Configuration](../assets/config/config.example.jsonc) - System settings

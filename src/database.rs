use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCode {
    pub id: Uuid,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by_user_id: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub invite_code_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebauthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub credential_data: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Invite code operations
    pub async fn create_invite_code(&self, code: &str) -> Result<InviteCode, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO invite_codes (code)
            VALUES ($1)
            RETURNING id, code, created_at, used_at, used_by_user_id, is_active
            "#,
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await?;

        Ok(InviteCode {
            id: row.get("id"),
            code: row.get("code"),
            created_at: row.get("created_at"),
            used_at: row.get("used_at"),
            used_by_user_id: row.get("used_by_user_id"),
            is_active: row.get("is_active"),
        })
    }

    pub async fn get_invite_code(&self, code: &str) -> Result<Option<InviteCode>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, code, created_at, used_at, used_by_user_id, is_active
            FROM invite_codes
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| InviteCode {
            id: r.get("id"),
            code: r.get("code"),
            created_at: r.get("created_at"),
            used_at: r.get("used_at"),
            used_by_user_id: r.get("used_by_user_id"),
            is_active: r.get("is_active"),
        }))
    }

    pub async fn use_invite_code(&self, code: &str, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE invite_codes
            SET used_at = NOW(), used_by_user_id = $2, is_active = FALSE
            WHERE code = $1 AND is_active = TRUE AND used_at IS NULL
            "#,
        )
        .bind(code)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_invite_codes(&self) -> Result<Vec<InviteCode>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, created_at, used_at, used_by_user_id, is_active
            FROM invite_codes
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| InviteCode {
                id: r.get("id"),
                code: r.get("code"),
                created_at: r.get("created_at"),
                used_at: r.get("used_at"),
                used_by_user_id: r.get("used_by_user_id"),
                is_active: r.get("is_active"),
            })
            .collect())
    }

    // User operations
    pub async fn create_user(
        &self,
        username: &str,
        invite_code: Option<&str>,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (username, invite_code_used)
            VALUES ($1, $2)
            RETURNING id, username, created_at, invite_code_used
            "#,
        )
        .bind(username)
        .bind(invite_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            invite_code_used: row.get("invite_code_used"),
        })
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, created_at, invite_code_used
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            created_at: r.get("created_at"),
            invite_code_used: r.get("invite_code_used"),
        }))
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, created_at, invite_code_used
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            created_at: r.get("created_at"),
            invite_code_used: r.get("invite_code_used"),
        }))
    }

    // WebAuthn credential operations
    pub async fn save_credential(
        &self,
        user_id: Uuid,
        passkey: &Passkey,
    ) -> Result<(), sqlx::Error> {
        let credential_id = passkey.cred_id().as_ref().to_vec();
        let credential_data =
            serde_json::to_string(passkey).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"
            INSERT INTO webauthn_credentials (user_id, credential_id, credential_data)
            VALUES ($1, $2, $3)
            ON CONFLICT (credential_id)
            DO UPDATE SET credential_data = $3, last_used_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(credential_id)
        .bind(credential_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_credentials(&self, user_id: Uuid) -> Result<Vec<Passkey>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT credential_data
            FROM webauthn_credentials
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut credentials = Vec::new();
        for row in rows {
            let credential_data: String = row.get("credential_data");
            match serde_json::from_str::<Passkey>(&credential_data) {
                Ok(passkey) => credentials.push(passkey),
                Err(e) => {
                    tracing::error!("Failed to deserialize passkey: {}", e);
                }
            }
        }

        Ok(credentials)
    }

    pub async fn update_credential(
        &self,
        user_id: Uuid,
        passkey: &Passkey,
    ) -> Result<(), sqlx::Error> {
        let credential_id = passkey.cred_id().as_ref().to_vec();
        let credential_data =
            serde_json::to_string(passkey).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;

        sqlx::query(
            r#"
            UPDATE webauthn_credentials
            SET credential_data = $3, last_used_at = NOW()
            WHERE user_id = $1 AND credential_id = $2
            "#,
        )
        .bind(user_id)
        .bind(credential_id)
        .bind(credential_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Utility method to run migrations
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        // This is a simple migration runner. In production, you might want to use sqlx-cli

        // Check if tables already exist to avoid duplicate creation
        let table_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'invite_codes')"
        )
        .fetch_one(&self.pool)
        .await?;

        if table_exists {
            return Ok(());
        }

        // Execute each migration statement individually
        let statements = vec![
            // Invite codes table
            r#"CREATE TABLE invite_codes (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                code VARCHAR(8) NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                used_at TIMESTAMPTZ,
                used_by_user_id UUID,
                is_active BOOLEAN NOT NULL DEFAULT TRUE
            )"#,

            "CREATE INDEX idx_invite_codes_code ON invite_codes(code)",
            "CREATE INDEX idx_invite_codes_active ON invite_codes(is_active) WHERE is_active = TRUE",

            // Users table
            r#"CREATE TABLE users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                invite_code_used VARCHAR(8),
                FOREIGN KEY (invite_code_used) REFERENCES invite_codes(code)
            )"#,

            "CREATE INDEX idx_users_username ON users(username)",

            // WebAuthn credentials table
            r#"CREATE TABLE webauthn_credentials (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                credential_id BYTEA NOT NULL UNIQUE,
                credential_data TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_used_at TIMESTAMPTZ
            )"#,

            "CREATE INDEX idx_webauthn_credentials_user_id ON webauthn_credentials(user_id)",
            "CREATE INDEX idx_webauthn_credentials_credential_id ON webauthn_credentials(credential_id)",

            // Sessions table
            r#"CREATE TABLE tower_sessions (
                id TEXT PRIMARY KEY,
                data BYTEA NOT NULL,
                expiry_date TIMESTAMPTZ NOT NULL
            )"#,

            "CREATE INDEX idx_tower_sessions_expiry ON tower_sessions(expiry_date)",
        ];

        for statement in statements {
            sqlx::query(statement).execute(&self.pool).await?;
        }

        Ok(())
    }
}

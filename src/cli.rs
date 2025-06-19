use crate::database::Database;
use clap::{Parser, Subcommand};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;

#[derive(Parser)]
#[command(name = "webauthn-admin")]
#[command(about = "WebAuthn administration CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new invite code
    GenerateInvite {
        /// Number of invite codes to generate
        #[arg(short, long, default_value = "1")]
        count: u32,
        /// Length of the invite code
        #[arg(short, long, default_value = "8")]
        length: usize,
    },
    /// List all invite codes
    ListInvites {
        /// Show only active invite codes
        #[arg(short, long)]
        active_only: bool,
    },
    /// Show invite code statistics
    Stats,
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgPool::connect(&self.database_url).await?;
        let db = Database::new(pool);

        // Run migrations
        db.migrate().await?;

        match self.command {
            Commands::GenerateInvite { count, length } => {
                self.generate_invites(&db, count, length).await?;
            }
            Commands::ListInvites { active_only } => {
                self.list_invites(&db, active_only).await?;
            }
            Commands::Stats => {
                self.show_stats(&db).await?;
            }
        }

        Ok(())
    }

    async fn generate_invites(
        &self,
        db: &Database,
        count: u32,
        length: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Generating {} invite code(s) of length {}...",
            count, length
        );
        println!();

        for i in 1..=count {
            let code = generate_invite_code(length);
            match db.create_invite_code(&code).await {
                Ok(invite_code) => {
                    println!(
                        "Generated invite code {}/{}: {}",
                        i, count, invite_code.code
                    );
                }
                Err(e) => {
                    eprintln!("Failed to generate invite code {}/{}: {}", i, count, e);
                }
            }
        }

        println!();
        println!("Done! Generated {} invite code(s).", count);
        Ok(())
    }

    async fn list_invites(
        &self,
        db: &Database,
        active_only: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let invite_codes = db.list_invite_codes().await?;

        let filtered_codes: Vec<_> = if active_only {
            invite_codes
                .into_iter()
                .filter(|code| code.is_active)
                .collect()
        } else {
            invite_codes
        };

        if filtered_codes.is_empty() {
            println!("No invite codes found.");
            return Ok(());
        }

        println!("Invite Codes:");
        println!(
            "{:<10} {:<8} {:<20} {:<20} {:<10}",
            "Code", "Active", "Created", "Used", "User ID"
        );
        println!("{}", "-".repeat(80));

        for code in filtered_codes {
            let used_at = code
                .used_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Never".to_string());

            let user_id = code
                .used_by_user_id
                .map(|id| id.to_string()[..8].to_string())
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:<10} {:<8} {:<20} {:<20} {:<10}",
                code.code,
                if code.is_active { "Yes" } else { "No" },
                code.created_at.format("%Y-%m-%d %H:%M"),
                used_at,
                user_id
            );
        }

        Ok(())
    }

    async fn show_stats(&self, db: &Database) -> Result<(), Box<dyn std::error::Error>> {
        let invite_codes = db.list_invite_codes().await?;

        let total_codes = invite_codes.len();
        let active_codes = invite_codes.iter().filter(|code| code.is_active).count();
        let used_codes = invite_codes
            .iter()
            .filter(|code| code.used_at.is_some())
            .count();

        println!("Invite Code Statistics:");
        println!("  Total codes: {}", total_codes);
        println!("  Active codes: {}", active_codes);
        println!("  Used codes: {}", used_codes);
        println!("  Unused codes: {}", total_codes - used_codes);

        Ok(())
    }
}

fn generate_invite_code(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_code() {
        let code = generate_invite_code(8);
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_alphanumeric()));
        assert!(code.chars().all(|c| c.is_uppercase() || c.is_numeric()));
    }

    #[test]
    fn test_generate_invite_code_different_lengths() {
        for length in [4, 6, 8, 12] {
            let code = generate_invite_code(length);
            assert_eq!(code.len(), length);
        }
    }
}

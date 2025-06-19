use crate::analytics::AnalyticsService;
use crate::database::Database;
use clap::{Parser, Subcommand};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;

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
    /// Show request analytics
    Analytics {
        /// Time period in hours
        #[arg(long, default_value = "24")]
        hours: i32,
        /// Number of top paths to show
        #[arg(long, default_value = "10")]
        limit: i64,
    },
    /// Show user request history
    UserActivity {
        /// User ID to look up
        #[arg(long)]
        user_id: String,
        /// Number of recent requests to show
        #[arg(long, default_value = "20")]
        limit: i64,
    },
    /// Clean up old analytics data
    CleanupAnalytics {
        /// Days of data to keep
        #[arg(long, default_value = "30")]
        days: i32,
        /// Actually perform the cleanup (dry run by default)
        #[arg(long)]
        execute: bool,
    },
}

impl Cli {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgPool::connect(&self.database_url).await?;
        let db = Database::new(pool.clone());

        // Run migrations
        db.migrate().await?;

        // Create analytics service
        let analytics = AnalyticsService::new(pool);

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
            Commands::Analytics { hours, limit } => {
                self.show_analytics(&analytics, hours, limit).await?;
            }
            Commands::UserActivity { ref user_id, limit } => {
                self.show_user_activity(&analytics, user_id, limit).await?;
            }
            Commands::CleanupAnalytics { days, execute } => {
                self.cleanup_analytics(&analytics, days, execute).await?;
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

    async fn show_analytics(
        &self,
        analytics: &AnalyticsService,
        hours: i32,
        limit: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Request Analytics (Last {} hours):", hours);
        println!("{}", "=".repeat(50));

        // Get overall stats
        let stats = analytics.get_stats(hours).await?;
        println!("📊 Overall Statistics:");
        println!("  Total Requests: {}", stats.total_requests);
        println!("  Unique Users: {}", stats.unique_users);
        println!(
            "  Success Rate: {:.1}%",
            if stats.total_requests > 0 {
                (stats.success_count as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "  Average Duration: {:.1}ms",
            stats.avg_duration_ms.unwrap_or(0.0)
        );
        println!("  Error Count: {}", stats.error_count);
        println!();

        // Get top paths
        let top_paths = analytics.get_top_paths(hours, limit).await?;
        if !top_paths.is_empty() {
            println!("🔥 Top Paths:");
            println!("{:<40} {:<10} {:<15}", "Path", "Requests", "Avg Duration");
            println!("{}", "-".repeat(70));
            for path_stat in top_paths {
                println!(
                    "{:<40} {:<10} {:<15.1}ms",
                    path_stat.path,
                    path_stat.request_count,
                    path_stat.avg_duration_ms.unwrap_or(0.0)
                );
            }
        } else {
            println!("No requests found in the specified time period.");
        }

        Ok(())
    }

    async fn show_user_activity(
        &self,
        analytics: &AnalyticsService,
        user_id_str: &str,
        limit: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let user_id = Uuid::parse_str(user_id_str)?;

        println!("User Activity for {}:", user_id);
        println!("{}", "=".repeat(60));

        let requests = analytics.get_user_requests(user_id, limit).await?;

        if requests.is_empty() {
            println!("No requests found for this user.");
            return Ok(());
        }

        println!(
            "{:<20} {:<8} {:<30} {:<6} {:<10}",
            "Timestamp", "Method", "Path", "Status", "Duration"
        );
        println!("{}", "-".repeat(80));

        for req in requests {
            println!(
                "{:<20} {:<8} {:<30} {:<6} {:<10}ms",
                req.timestamp.format("%Y-%m-%d %H:%M:%S"),
                req.method,
                if req.path.len() > 28 {
                    format!("{}...", &req.path[..25])
                } else {
                    req.path.clone()
                },
                req.status_code,
                req.duration_ms.unwrap_or(0)
            );
        }

        Ok(())
    }

    async fn cleanup_analytics(
        &self,
        analytics: &AnalyticsService,
        days: i32,
        execute: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if execute {
            println!("🗑️  Cleaning up analytics data older than {} days...", days);
            let deleted_count = analytics.cleanup_old_data(days).await?;
            println!(
                "✅ Successfully deleted {} old analytics records.",
                deleted_count
            );
        } else {
            println!(
                "🔍 Dry run: Would clean up analytics data older than {} days",
                days
            );
            println!("   Use --execute to actually perform the cleanup.");
        }

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

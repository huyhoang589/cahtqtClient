pub mod logs_repo;
pub mod partner_members_repo;
pub mod partners_repo;
pub mod settings_repo;

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;

/// Initialize the SQLite connection pool and run schema migrations.
/// E: DB is now at DATA/DB/cahtqt.db; auto-migrates from older locations on first run.
pub async fn init_db(app_data_dir: &Path) -> Result<Pool<Sqlite>, sqlx::Error> {
    // E: new DB location — DATA/DB/cahtqt.db
    let db_dir = app_data_dir.join("DATA").join("DB");
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| sqlx::Error::Configuration(e.into()))?;

    let new_db_path = db_dir.join("cahtqt.db");

    // E: auto-migrate from old locations (copy first found; old file stays for manual recovery)
    if !new_db_path.exists() {
        let old_locations = [
            app_data_dir.join("DATA").join("partners.db"),  // v3 intermediate
            app_data_dir.join("cahtqt.db"),                 // original root
        ];
        for old in &old_locations {
            if old.exists() {
                let _ = std::fs::copy(old, &new_db_path);
                break;
            }
        }
    }

    let db_url = format!("sqlite://{}?mode=rwc", new_db_path.to_string_lossy());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

/// Versioned migration runner using PRAGMA user_version for idempotency.
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;

    // Migration 001 — initial schema (CREATE TABLE IF NOT EXISTS is idempotent)
    if version < 1 {
        let sql = include_str!("../../migrations/001_init.sql");
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(pool).await?;
            }
        }
        sqlx::query("PRAGMA user_version = 1").execute(pool).await?;
    }

    // Migration 002 — rename tables + columns (groups→partners, recipients→partner_members)
    if version < 2 {
        let sql = include_str!("../../migrations/002_rename_tables.sql");
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                // Ignore errors from statements that already ran (e.g., "no such table")
                let _ = sqlx::query(trimmed).execute(pool).await;
            }
        }
        sqlx::query("PRAGMA user_version = 2").execute(pool).await?;
    }

    // Migration 003 — rename settings keys for eToken module naming alignment
    if version < 3 {
        let sql = include_str!("../../migrations/003_migrate_settings_keys.sql");
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                let _ = sqlx::query(trimmed).execute(pool).await;
            }
        }
        sqlx::query("PRAGMA user_version = 3").execute(pool).await?;
    }

    // Migration 004 — add cert_org column to partner_members if missing (older DBs)
    if version < 4 {
        let sql = include_str!("../../migrations/004_add_cert_org.sql");
        for stmt in sql.split(';') {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                let _ = sqlx::query(trimmed).execute(pool).await;
            }
        }
        sqlx::query("PRAGMA user_version = 4").execute(pool).await?;
    }

    Ok(())
}

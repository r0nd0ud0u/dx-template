#[cfg(feature = "server")]
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus::server::ServerFnError;
#[cfg(feature = "server")]
use sqlx::sqlite::SqliteConnectOptions;
#[cfg(feature = "server")]
use sqlx::{Executor, Pool, Sqlite};
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use tokio::sync::OnceCell;

#[cfg(feature = "server")]
static DB: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

#[cfg(feature = "server")]
use dioxus::logger::tracing;

#[cfg(feature = "server")]
async fn db() -> Pool<Sqlite> {
    let db_url = match get_db_url().await {
        Ok(url) => url,
        Err(e) => {
            panic!("Failed to get database URL: {}", e);
        }
    };

    // Ensure the parent directory and the file exist before connecting.
    // SQLite cannot create missing parent directories on its own.
    if let Some(path_str) = db_url.strip_prefix("sqlite://") {
        let path = std::path::Path::new(path_str);
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            tracing::warn!("Could not create database directory {:?}: {}", parent, e);
        }
    }

    let connect_opts = match SqliteConnectOptions::from_str(&db_url) {
        Ok(opts) => opts.create_if_missing(true),
        Err(e) => panic!("Invalid DATABASE_URL: {}", e),
    };

    let pool = match sqlx::sqlite::SqlitePool::connect_with(connect_opts).await {
        Ok(pool) => pool,
        Err(e) => {
            panic!("Failed to connect to database: {}", e);
        }
    };

    // Create the tables (sessions, users)
    pool.execute(r#"CREATE TABLE IF NOT EXISTS users ( "id" INTEGER PRIMARY KEY, "anonymous" BOOLEAN NOT NULL, "username" VARCHAR(256) NOT NULL, "password" VARCHAR(256), "is_connected" BOOLEAN NOT NULL)"#,)
            .await.unwrap();
    pool.execute(r#"CREATE TABLE IF NOT EXISTS user_permissions ( "user_id" INTEGER NOT NULL, "token" VARCHAR(256) NOT NULL)"#,)
            .await.unwrap();
    pool.execute(r#"CREATE TABLE IF NOT EXISTS user_settings ( "username" VARCHAR(256) NOT NULL, "key" VARCHAR(64) NOT NULL, "value" VARCHAR(256) NOT NULL, PRIMARY KEY("username","key"))"#,)
            .await.unwrap();

    // Insert in some test data for two users (one anonymous, one normal)
    pool.execute(r#"INSERT INTO users (id, anonymous, username, password, is_connected) SELECT 1, true, 'Admin', '', false ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username, password = EXCLUDED.password, is_connected = EXCLUDED.is_connected"#,)
            .await.unwrap();
    pool.execute(r#"INSERT INTO users (id, anonymous, username, password, is_connected) SELECT 2, false, 'Guest', '', false ON CONFLICT(id) DO UPDATE SET anonymous = EXCLUDED.anonymous, username = EXCLUDED.username, password = EXCLUDED.password, is_connected = EXCLUDED.is_connected"#,)
            .await.unwrap();

    // permissions
    pool.execute(r#"INSERT INTO user_permissions (user_id, token) SELECT 1, 'Admin::View'"#)
        .await
        .unwrap();

    pool.execute(r#"INSERT INTO user_permissions (user_id, token) SELECT 2, 'Category::View'"#)
        .await
        .unwrap();

    pool
}

#[cfg(feature = "server")]
pub async fn get_db() -> &'static Pool<Sqlite> {
    DB.get_or_init(db).await
}

#[post("/api/get_db_url")]
async fn get_db_url() -> Result<String, ServerFnError> {
    Ok(std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db.sqlite".to_owned()))
}

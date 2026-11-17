use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub enum DbError {
    Pool(sqlx::Error),
    Migration(sqlx::migrate::MigrateError),
    Configuration(String),
    PragmaVerification(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pool(err) => write!(f, "Database pool error: {err}"),
            Self::Migration(err) => write!(f, "Migration error: {err}"),
            Self::Configuration(msg) => write!(f, "Database configuration error: {msg}"),
            Self::PragmaVerification(msg) => write!(f, "Pragma verification failed: {msg}"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pool(err) => Some(err),
            Self::Migration(err) => Some(err),
            Self::Configuration(_) | Self::PragmaVerification(_) => None,
        }
    }
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        Self::Pool(err)
    }
}

impl From<sqlx::migrate::MigrateError> for DbError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::Migration(err)
    }
}

/// Initializes an SQLite connection pool with production PRAGMAs:
/// - WAL journal mode: Readers read consistent snapshots and can observe committed WAL frames
///   applicable to their snapshot, but never uncommitted transactions. SQLite permits only one
///   active writer at any time. Committed frames in the WAL are periodically checkpointed back
///   to the database file; prolonged open read snapshots can delay checkpointing and increase WAL size.
/// - Foreign keys enabled: Strictly enforces referential integrity on every connection.
/// - 5000ms busy timeout: Configures SQLite to retry contended locks up to 5000ms before returning
///   `SQLITE_BUSY`, returning immediately if the lock clears earlier.
/// - NORMAL synchronous mode: Guarantees database structural consistency without corruption across
///   application crashes. In exchange for significant write performance gains, an abrupt power failure
///   or OS crash could cause the most recently committed transaction(s) to be lost if unwritten by the disk controller.
pub async fn init_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool, DbError> {
    if max_connections == 0 {
        return Err(DbError::Configuration(
            "max_connections must be at least 1".to_string(),
        ));
    }

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| DbError::Configuration(e.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await?;

    Ok(pool)
}

/// Verifies that essential SQLite pragmas are actively enforced.
pub async fn verify_pragmas(pool: &SqlitePool) -> Result<(), DbError> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;

    if !journal_mode.eq_ignore_ascii_case("wal") && !journal_mode.eq_ignore_ascii_case("memory") {
        return Err(DbError::PragmaVerification(format!(
            "Expected journal_mode 'wal', got: '{journal_mode}'"
        )));
    }

    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;

    if foreign_keys != 1 {
        return Err(DbError::PragmaVerification(format!(
            "Expected foreign_keys 1, got: {foreign_keys}"
        )));
    }

    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await?;

    if busy_timeout < 5000 {
        return Err(DbError::PragmaVerification(format!(
            "Expected busy_timeout >= 5000, got: {busy_timeout}"
        )));
    }

    Ok(())
}

/// Runs embedded database migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DbError::Migration)?;
    Ok(())
}

/// Checks database connectivity by executing a lightweight ping query.
pub async fn check_connection(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

/// Initializes the complete database foundation: pool, pragma verification, and migrations.
pub async fn init_db(database_url: &str, max_connections: u32) -> Result<SqlitePool, DbError> {
    let pool = init_pool(database_url, max_connections).await?;
    verify_pragmas(&pool).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

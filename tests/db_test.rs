use nexus_cme::api::{app_router, health::HealthResponse, AppState};
use nexus_cme::db::{
    check_connection, init_db, init_pool, run_migrations, verify_pragmas, DbError,
};
use tempfile::NamedTempFile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn test_db_init_and_connectivity() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_db(&db_url, 5).await.expect("init_db should succeed");
    check_connection(&pool)
        .await
        .expect("check_connection should succeed");

    // Verify migration tracking record created by SQLx
    let version: i64 =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE version = 20261114000000")
            .fetch_one(&pool)
            .await
            .expect("fetch migration record should succeed");
    assert_eq!(version, 20261114000000);

    pool.close().await;
}

#[tokio::test]
async fn test_repeated_initialization_is_idempotent() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_db(&db_url, 5)
        .await
        .expect("First init should succeed");
    // Run second init on the same file
    run_migrations(&pool)
        .await
        .expect("Second migration run should be idempotent");

    let version: i64 =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE version = 20261114000000")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(version, 20261114000000);

    pool.close().await;
}

#[tokio::test]
async fn test_wal_mode_and_pragmas_on_file_db() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_pool(&db_url, 2).await.unwrap();
    verify_pragmas(&pool).await.unwrap();

    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(foreign_keys, 1);

    let busy_timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(busy_timeout, 5000);

    pool.close().await;
}

#[tokio::test]
async fn test_connection_level_pragmas_on_multiple_connections() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_pool(&db_url, 5).await.unwrap();

    // Acquire multiple distinct connections concurrently
    let mut conn1 = pool.acquire().await.unwrap();
    let mut conn2 = pool.acquire().await.unwrap();
    let mut conn3 = pool.acquire().await.unwrap();

    for (idx, conn) in [&mut conn1, &mut conn2, &mut conn3].iter_mut().enumerate() {
        let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&mut ***conn)
            .await
            .unwrap();
        assert_eq!(fk, 1, "Connection {idx} foreign_keys was not 1");

        let timeout: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(&mut ***conn)
            .await
            .unwrap();
        assert_eq!(timeout, 5000, "Connection {idx} busy_timeout was not 5000");

        let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&mut ***conn)
            .await
            .unwrap();
        assert_eq!(
            journal.to_lowercase(),
            "wal",
            "Connection {idx} journal_mode was not wal"
        );
    }

    drop(conn1);
    drop(conn2);
    drop(conn3);
    pool.close().await;
}

#[tokio::test]
async fn test_fts5_runtime_verification() {
    let pool = init_pool("sqlite::memory:", 1).await.unwrap();

    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!version.is_empty());

    let fts5_option: i64 = sqlx::query_scalar("SELECT sqlite_compileoption_used('ENABLE_FTS5')")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        fts5_option, 1,
        "FTS5 must be enabled in the runtime SQLite build"
    );

    let create_result = sqlx::query("CREATE VIRTUAL TABLE temp.test_fts USING fts5(content);")
        .execute(&pool)
        .await;
    assert!(
        create_result.is_ok(),
        "Creating an FTS5 virtual table must succeed: {:?}",
        create_result.err()
    );

    pool.close().await;
}

#[tokio::test]
async fn test_foreign_key_enforcement() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_pool(&db_url, 1).await.unwrap();

    // Create related tables
    sqlx::query("CREATE TABLE parents (id INTEGER PRIMARY KEY);")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE children (id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parents(id));",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Valid insert
    sqlx::query("INSERT INTO parents (id) VALUES (1);")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO children (id, parent_id) VALUES (10, 1);")
        .execute(&pool)
        .await
        .unwrap();

    // Invalid insert: parent_id 999 does not exist
    let invalid_result = sqlx::query("INSERT INTO children (id, parent_id) VALUES (20, 999);")
        .execute(&pool)
        .await;

    // Use structured error checking instead of brittle string matching
    match invalid_result {
        Err(sqlx::Error::Database(ref db_err)) => {
            assert!(
                db_err.is_foreign_key_violation(),
                "Expected foreign key constraint violation error"
            );
            assert_eq!(db_err.code().as_deref(), Some("787")); // SQLITE_CONSTRAINT_FOREIGNKEY
        }
        other => panic!("Expected Err(sqlx::Error::Database), got: {other:?}"),
    }

    pool.close().await;
}

#[tokio::test]
async fn test_db_failure_handling_on_closed_pool() {
    let pool = init_pool("sqlite::memory:", 1).await.unwrap();
    pool.close().await;

    let res = check_connection(&pool).await;
    assert!(
        matches!(res, Err(DbError::Pool(sqlx::Error::PoolClosed))),
        "Expected DbError::Pool(sqlx::Error::PoolClosed), got: {res:?}"
    );
}

#[tokio::test]
async fn test_init_pool_zero_max_connections() {
    let res = init_pool("sqlite::memory:", 0).await;
    assert!(
        matches!(res, Err(DbError::Configuration(_))),
        "Expected DbError::Configuration when max_connections is 0, got: {res:?}"
    );
}

#[tokio::test]
async fn test_health_endpoint_integration_with_real_db() {
    let temp_db = NamedTempFile::new().unwrap();
    let db_url = format!("sqlite://{}", temp_db.path().to_str().unwrap());

    let pool = init_db(&db_url, 2).await.unwrap();
    let state = AppState { db: pool.clone() };
    let app = app_router(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Test healthy state
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).await.unwrap();

    assert!(resp.starts_with("HTTP/1.1 200 OK"));
    let body = resp.split("\r\n\r\n").nth(1).unwrap();
    let health: HealthResponse = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(health.status, "ok");
    assert_eq!(health.service, "nexus-cme");
    assert_eq!(health.database, "connected");

    // Close the pool and verify health degradation to 503
    pool.close().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut resp2 = String::new();
    stream.read_to_string(&mut resp2).await.unwrap();

    assert!(resp2.starts_with("HTTP/1.1 503 Service Unavailable"));
    let body2 = resp2.split("\r\n\r\n").nth(1).unwrap();
    let health2: HealthResponse = serde_json::from_str(body2.trim()).unwrap();
    assert_eq!(health2.status, "error");
    assert_eq!(health2.service, "nexus-cme");
    assert_eq!(health2.database, "disconnected");

    server_handle.abort();
}

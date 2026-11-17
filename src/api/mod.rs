pub mod health;

use axum::{routing::get, Router};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::health::HealthResponse;
    use crate::db::init_pool;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn test_health_endpoint_healthy() {
        let pool = init_pool("sqlite::memory:", 1).await.unwrap();
        let state = AppState { db: pool.clone() };
        let app = app_router(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();

        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "Expected 200 OK, got: {response}"
        );

        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .expect("Response missing body");

        let health_data: HealthResponse =
            serde_json::from_str(body.trim()).expect("Failed to deserialize response body");

        assert_eq!(health_data.status, "ok");
        assert_eq!(health_data.service, "nexus-cme");
        assert_eq!(health_data.database, "connected");

        server_handle.abort();
        pool.close().await;
    }

    #[tokio::test]
    async fn test_health_endpoint_unhealthy_when_db_closed() {
        let pool = init_pool("sqlite::memory:", 1).await.unwrap();
        // Close the pool to simulate database outage
        pool.close().await;

        let state = AppState { db: pool.clone() };
        let app = app_router(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();

        assert!(
            response.starts_with("HTTP/1.1 503 Service Unavailable"),
            "Expected 503 Service Unavailable, got: {response}"
        );

        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .expect("Response missing body");

        let health_data: HealthResponse =
            serde_json::from_str(body.trim()).expect("Failed to deserialize response body");

        assert_eq!(health_data.status, "error");
        assert_eq!(health_data.service, "nexus-cme");
        assert_eq!(health_data.database, "disconnected");

        server_handle.abort();
        pool.close().await;
    }
}

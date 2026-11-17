use crate::api::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub database: String,
}

pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    match crate::db::check_connection(&state.db).await {
        Ok(()) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok".to_string(),
                service: "nexus-cme".to_string(),
                database: "connected".to_string(),
            }),
        ),
        Err(err) => {
            tracing::error!(error = %err, "Database health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "error".to_string(),
                    service: "nexus-cme".to_string(),
                    database: "disconnected".to_string(),
                }),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_connected_serialization() {
        let response = HealthResponse {
            status: "ok".to_string(),
            service: "nexus-cme".to_string(),
            database: "connected".to_string(),
        };
        let serialized = serde_json::to_string(&response).expect("serialization failed");
        let expected = r#"{"status":"ok","service":"nexus-cme","database":"connected"}"#;
        assert_eq!(serialized, expected);

        let deserialized: HealthResponse =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(deserialized, response);
    }

    #[test]
    fn test_health_response_disconnected_serialization() {
        let response = HealthResponse {
            status: "error".to_string(),
            service: "nexus-cme".to_string(),
            database: "disconnected".to_string(),
        };
        let serialized = serde_json::to_string(&response).expect("serialization failed");
        let expected = r#"{"status":"error","service":"nexus-cme","database":"disconnected"}"#;
        assert_eq!(serialized, expected);

        let deserialized: HealthResponse =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(deserialized, response);
    }
}

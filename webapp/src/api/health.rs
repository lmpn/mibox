use axum::{http::StatusCode, response::IntoResponse};

pub async fn health_check_service_handler() -> impl IntoResponse {
    StatusCode::OK
}

use axum::http::StatusCode;
use axum::response::IntoResponse;
pub async fn health_check() -> impl IntoResponse {
    tracing::info!("health_check");
    StatusCode::OK
}

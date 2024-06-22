use axum::{http::StatusCode, response::IntoResponse};

pub async fn fallback_service_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

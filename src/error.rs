use axum::http::StatusCode;
use axum::response::IntoResponse;

pub struct MiboxError(pub StatusCode, pub anyhow::Error);

impl IntoResponse for MiboxError {
    fn into_response(self) -> axum::response::Response {
        (
            self.0,
            format!("Error: {}", self.1),
        )
            .into_response()
    }
}

impl<E> From<E> for MiboxError
    where E : Into<anyhow::Error>
{
    fn from(value: E) -> Self {
        MiboxError(StatusCode::INTERNAL_SERVER_ERROR, value.into())
    }
}

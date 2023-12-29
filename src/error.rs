use axum::http::StatusCode;
use axum::response::IntoResponse;

pub struct MiboxError(anyhow::Error);

impl IntoResponse for MiboxError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for MiboxError
    where E : Into<anyhow::Error>
{
    fn from(value: E) -> Self {
        MiboxError(value.into())
    }
}

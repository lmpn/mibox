use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::{header::WWW_AUTHENTICATE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

#[derive(thiserror::Error)]
pub enum NewsletterError {
    #[error("{0}")]
    ValidationError(String),
    #[error("Authentication error")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
}

impl std::fmt::Debug for NewsletterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

impl IntoResponse for NewsletterError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);

        match self {
            NewsletterError::JsonRejection(_) => {
                (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
            }
            NewsletterError::AuthError(_) => {
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                let mut header_map = HeaderMap::new();
                header_map.insert(WWW_AUTHENTICATE, header_value);
                (StatusCode::UNAUTHORIZED, header_map, format!("{}", self)).into_response()
            }
            NewsletterError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, "Bad request".to_owned()).into_response()
            }
            NewsletterError::UnexpectedError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_owned(),
            )
                .into_response(),
        }
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(NewsletterError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

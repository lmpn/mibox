use crate::error::MiboxError;
use anyhow::anyhow;
use axum::{body::Body, extract::Query, response::IntoResponse};
use hyper::{header, StatusCode};
use mime_guess::mime::TEXT_PLAIN_UTF_8;
use tokio_util::io::ReaderStream;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn download_service_handler(
    Query(QueryParams { path }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => {
            return Err(MiboxError(
                StatusCode::NOT_FOUND,
                anyhow!("File not found: {}", err),
            ))
        }
    };
    let content_type = mime_guess::from_path(&path)
        .first_raw()
        .unwrap_or_else(|| TEXT_PLAIN_UTF_8.as_ref());

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [(header::CONTENT_TYPE, content_type)];

    Ok((headers, body))
}

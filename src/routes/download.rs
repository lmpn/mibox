use anyhow::Context;
use axum::{body::Body, extract::Query, http::StatusCode, response::IntoResponse};
use tokio_util::io::ReaderStream;

use crate::{domain::drive::Drive, error::MiboxError};

#[derive(serde::Deserialize)]
pub struct DownloadQueryParams {
    path: String,
}

#[tracing::instrument(name = "Download file", skip(), fields())]
pub async fn download(
    Query(DownloadQueryParams { path }): Query<DownloadQueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let file = Drive::new("tmp".to_string())
        .download(path)
        .await
        .context("failed to download file")?;
    let headers = [(
        axum::http::header::CONTENT_TYPE,
        file.content_type().to_owned(),
    )];

    let stream = ReaderStream::from(file);
    let body = Body::from_stream(stream);
    Ok((StatusCode::OK, headers, body))
}

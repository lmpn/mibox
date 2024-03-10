use std::io;

use anyhow::Context;
use axum::{
    extract::{Query, Request},
    http::StatusCode,
    response::IntoResponse,
};
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

use crate::{domain::drive::Drive, error::MiboxError};

#[derive(serde::Deserialize)]
pub struct UploadQueryParams {
    path: String,
}

#[tracing::instrument(name = "Upload file", skip(), fields())]
pub async fn upload(
    Query(UploadQueryParams { path }): Query<UploadQueryParams>,
    request: Request,
) -> Result<impl IntoResponse, MiboxError> {
    let body = request
        .into_body()
        .into_data_stream()
        .into_stream()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    let stream = StreamReader::new(body);
    Drive::new("tmp".to_string())
        .upload(path, stream)
        .await
        .context("failed to upload file")?;
    Ok(StatusCode::OK)
}

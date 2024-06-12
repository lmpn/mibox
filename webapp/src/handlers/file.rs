use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    body::Body,
    debug_handler,
    extract::{Multipart, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use futures::TryStreamExt;
use serde::Deserialize;
use std::io;

#[derive(Debug, Deserialize)]
pub struct DeleteParameters {
    path: String,
}

#[tracing::instrument(name = "File delete", skip(application))]
#[debug_handler]
pub async fn delete_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<DeleteParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    let path = application.drive.join(params.path.clone());
    tokio::fs::remove_file(path.clone())
        .await
        .context(format!("error removing file {:?}", path))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct DownloadParameters {
    path: String,
}

#[tracing::instrument(name = "File download", skip(application))]
#[debug_handler]
pub async fn download_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<DownloadParameters>, MiboxError>,
) -> Result<impl IntoResponse, MiboxError> {
    let d = Drive::new(application.drive)
        .read(params.path.clone())
        .await
        .context("file stream")?;
    let body = Body::from_stream(d);

    let headers = [
        (header::CONTENT_TYPE, "text/toml; charset=utf-8".to_owned()),
        (
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}\"",
                params.path.split('/').last().unwrap_or("")
            ),
        ),
    ];

    return Ok((headers, body));
}

#[derive(Debug, Deserialize)]
pub struct UploadParameters {
    path: String,
}

#[tracing::instrument(name = "File upload", skip(application, multipart))]
pub async fn upload_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<UploadParameters>, MiboxError>,
    mut multipart: Multipart,
) -> Result<StatusCode, MiboxError> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };
        let stream = field.map_err(|e| io::Error::new(io::ErrorKind::Other, e));
        let path = application.drive.join(params.path.clone());
        Drive::new(path.clone())
            .write(stream, file_name)
            .await
            .context("error uploading file")?;
    }
    Ok(StatusCode::OK)
}

use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    body::Body,
    debug_handler,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use tokio_util::io::ReaderStream;

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
    let path = application.drive.join(params.path.clone());
    let file = tokio::fs::File::open(path.clone())
        .await
        .context(format!("error opening file {:?}", path))?;

    if file.metadata().await.context("no metadata")?.is_file() {
        let reader = ReaderStream::new(file);
        let body = Body::from_stream(reader);

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
    Err(MiboxError::ValidationError("invalid path".to_owned()))
}

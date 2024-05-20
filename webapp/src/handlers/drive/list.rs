use crate::{application::Application, error::MiboxError};
use anyhow::{anyhow, Context};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::{header::ACCEPT, HeaderMap},
    Json,
};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use tokio::fs::read_dir;

#[derive(Debug, Deserialize)]
pub struct ListParameters {
    path: String,
}

#[derive(Debug, Serialize)]
pub struct DriveView {
    path: String,
    is_directory: bool,
}

#[tracing::instrument(name = "Drive listing", skip(application, headers))]
#[debug_handler]
pub async fn list_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
    headers: HeaderMap,
) -> Result<Json<Vec<DriveView>>, MiboxError> {
    let path = application.drive.join(params.path.clone());
    path.canonicalize().context("non-canonical path")?;
    let mut directory = read_dir(path).await.context("cannot read dir")?;
    let mut view = vec![];
    while let Some(entry) = directory.next_entry().await.context("error iterating")? {
        let is_dir = !entry.metadata().await.context("metadata error")?.is_file();
        let path = entry
            .path()
            .into_os_string()
            .to_str()
            .unwrap_or("")
            .to_string();
        let drive = DriveView {
            is_directory: is_dir,
            path,
        };
        view.push(drive);
    }
    let accept_header = headers.get(ACCEPT).context("no accept header")?;
    let accept_header = accept_header.to_str().context("invalid accept header")?;
    if accept_header.contains("*/*") || accept_header.contains("application/json") {
        return Ok(axum::Json(view));
    }
    return Err(MiboxError::UnexpectedError(anyhow!(
        "invalid accept header value"
    )));
}

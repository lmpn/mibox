use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CreateDirParameters {
    path: String,
}

#[tracing::instrument(name = "Create directory", skip(application))]
#[debug_handler]
pub async fn create_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<CreateDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    Drive::new(application.drive)
        .create_directory(params.path)
        .await
        .context("create")?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ListParameters {
    path: String,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct EntryView {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct DirectoryView {
    pub results: Vec<EntryView>,
}

#[tracing::instrument(name = "Directory listing", skip(application))]
#[debug_handler]
pub async fn list_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
) -> Result<impl IntoResponse, MiboxError> {
    let entries = Drive::new(application.drive)
        .entries(&params.path)
        .await
        .context("error listing directory")?;

    let view = entries
        .iter()
        .map(|elem| {
            let name = elem.file_name();
            let path = if params.path.is_empty() {
                name.clone()
            } else {
                format!("{}/{name}", params.path)
            };

            Some(EntryView {
                is_directory: elem.is_directory(),
                path,
                name,
            })
        })
        .filter(Option::is_some)
        .flatten()
        .collect::<Vec<EntryView>>();
    let view = DirectoryView { results: view };
    let raw = json!(view);
    let json = serde_json::to_value(raw).context("error serializing response")?;
    Ok(axum::Json(json))
}

#[derive(Debug, Deserialize)]
pub struct RemoveDirParameters {
    path: String,
}

#[tracing::instrument(name = "Remove directory", skip(application))]
#[debug_handler]
pub async fn remove_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<RemoveDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    let path = application.drive.join(params.path.clone());
    tokio::fs::remove_dir_all(path.clone())
        .await
        .context(format!("error removing directory {:?}", path))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct UpdateDirParameters {
    from: String,
    to: String,
}

#[tracing::instrument(name = "Update directory", skip(application))]
#[debug_handler]
pub async fn update_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<UpdateDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    Drive::new(application.drive)
        .rename_directory(params.from, params.to)
        .await
        .context("rename")?;

    Ok(StatusCode::NO_CONTENT)
}

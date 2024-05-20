use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RemoveDirParameters {
    path: String,
}

#[tracing::instrument(name = "Remove directory delete", skip(application))]
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

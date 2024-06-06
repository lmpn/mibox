use crate::{application::Application, drive::Drive, error::MiboxError};
use anyhow::Context;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use serde::Deserialize;

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

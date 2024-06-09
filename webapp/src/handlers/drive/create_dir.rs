use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::Deserialize;

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

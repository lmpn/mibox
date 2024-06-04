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
    //TODO handle the path verification
    let from = application.drive.join(params.from.clone());
    let to = application.drive.join(params.to.clone());
    tokio::fs::rename(from.clone(), to.clone())
        .await
        .context(format!("error renaming directory {:?} -> {:?}", from, to))?;

    Ok(StatusCode::NO_CONTENT)
}

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

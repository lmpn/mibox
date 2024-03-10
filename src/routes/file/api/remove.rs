use anyhow::Context;
use axum::{extract::Query, http::StatusCode, response::IntoResponse};

use crate::{domain::drive::Drive, error::MiboxError};

#[derive(serde::Deserialize)]
pub struct RemoveQueryParams {
    path: String,
}

#[tracing::instrument(name = "Delete file", skip(), fields())]
pub async fn remove(
    Query(RemoveQueryParams { path }): Query<RemoveQueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    Drive::new("tmp".to_string())
        .remove(path)
        .await
        .context("failed to delete file")?;
    Ok(StatusCode::OK)
}

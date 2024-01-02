use crate::core::drive::Drive;
use crate::axum_server::error::MiboxError;
use anyhow::anyhow;
use axum::{extract::Query, response::IntoResponse};
use hyper::StatusCode;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn removal_service_handler(
    Query(QueryParams { path }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    Drive::default().delete(path.clone()).await.map_err(|err| {
        tracing::error!("Error deleting file {:?} : {}", path, err);
        MiboxError(
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow!("File could not be deleted"),
        )
    })
}

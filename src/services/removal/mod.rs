use std::path::Component;

use crate::server::error::MiboxError;
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
    let path = std::path::Path::new(&path);
    let is_valid = path
        .components()
        .filter(|cmp| {
            cmp == &Component::ParentDir || cmp == &Component::RootDir || cmp == &Component::CurDir
        })
        .count()
        > 0;
    if is_valid {
        tracing::error!("tree walk detected {:?}", path);
        return Err(MiboxError(
            StatusCode::BAD_REQUEST,
            anyhow!("File name is invalid"),
        ));
    }
    //TODO use state
    let path = std::path::Path::new(crate::server::DRIVE_DIRECTORY)
        .join(path)
        .canonicalize()
        .map_err(|err| {
            tracing::error!("Non-canonical path {:?} : {}", path, err);
            MiboxError(StatusCode::BAD_REQUEST, anyhow!("File not found"))
        })?;

    tokio::fs::remove_file(&path)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|err| {
            tracing::error!("Non-canonical path {:?} : {}", path, err);
            MiboxError(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("File could not be deleted"),
            )
        })
}

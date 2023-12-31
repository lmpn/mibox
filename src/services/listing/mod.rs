use anyhow::anyhow;
use axum::{extract::Query, response::IntoResponse};
use hyper::StatusCode;
use std::ffi::OsStr;
use tracing::error;

use crate::error::MiboxError;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    page: usize,
    count: usize,
}

pub async fn listing_service_handler(
    Query(QueryParams { page, count }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let path = std::path::Path::new(crate::server::DRIVE_DIRECTORY);
    let files: String = itertools::Itertools::intersperse(
        path.read_dir()
            .map_err(|err| {
                error!("{}", err);
                MiboxError(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    anyhow!("error loading files"),
                )
            })?
            .skip(page * count)
            .take(count)
            .filter_map(Result::ok)
            .filter_map(|entry| entry.path().file_name().map(OsStr::to_os_string))
            .filter_map(|entry| entry.to_str().map(str::to_string)),
        ",".to_string(),
    )
    .collect();
    Ok((StatusCode::OK, files))
}

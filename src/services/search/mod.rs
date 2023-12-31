use axum::{extract::Query, response::IntoResponse};
use hyper::StatusCode;
use itertools::Itertools;
use std::ffi::OsStr;

use crate::error::MiboxError;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    query: String,
    page: usize,
    count: usize,
}

pub async fn search_service_handler(
    Query(QueryParams { query, page, count }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let path = std::path::Path::new(crate::server::DRIVE_DIRECTORY);
    let files = path
        .read_dir()
        .map_err(|err| {
            tracing::error!("{}", err);
            MiboxError(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow::anyhow!("error loading files"),
            )
        })?
        .filter_map(Result::ok)
        .sorted_by(|a, b| Ord::cmp(&a.path(), &b.path()))
        .filter_map(|entry| entry.path().file_name().map(OsStr::to_os_string))
        .filter_map(|entry| entry.to_str().map(str::to_string))
        .filter(|entry| entry.starts_with(&query))
        .skip(page * count)
        .take(count);
    let files: String = itertools::Itertools::intersperse(files, ",".to_string()).collect();
    Ok((StatusCode::OK, files))
}

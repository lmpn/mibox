use std::path::Component;

use crate::error::MiboxError;
use anyhow::anyhow;
use axum::{body::Body, extract::Query, response::IntoResponse};
use hyper::{header, StatusCode};
use mime_guess::mime::TEXT_PLAIN_UTF_8;
use tokio_util::io::ReaderStream;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn download_service_handler(
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

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file {:?}: {}", path, err);
            return Err(MiboxError(StatusCode::NOT_FOUND, anyhow!("File not found")));
        }
    };
    let content_type = mime_guess::from_path(&path)
        .count()
        .first_raw()
        .unwrap_or_else(|| TEXT_PLAIN_UTF_8.as_ref());

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [(header::CONTENT_TYPE, content_type)];

    Ok((headers, body))
}

use crate::error::MiboxError;
use anyhow::anyhow;
use axum::{
    extract::{Query, Request},
    response::IntoResponse,
};
use futures::TryStreamExt;
use hyper::StatusCode;
use std::{io, path::Component};
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn upload_service_handler(
    Query(QueryParams { path }): Query<QueryParams>,
    request: Request,
) -> Result<impl IntoResponse, MiboxError> {
    let body = request
        .into_body()
        .into_data_stream()
        .into_stream()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    let mut stream_reader = StreamReader::new(body);

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

    let file = tokio::fs::File::create(path).await.map_err(|err| {
        tracing::error!("{}", err);
        MiboxError(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("{}", err))
    });
    let mut buffer = BufWriter::new(file?);

    tokio::io::copy(&mut stream_reader, &mut buffer)
        .await
        .map(|_| (StatusCode::CREATED))
        .map_err(|err| {
            tracing::error!("{}", err);
            MiboxError(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("{}", err))
        })
}

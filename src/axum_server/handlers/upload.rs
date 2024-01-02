use crate::{core::drive::Drive, axum_server::error::MiboxError};
use axum::{
    extract::{Query, Request},
    response::IntoResponse,
};
use futures::TryStreamExt;
use hyper::StatusCode;
use std::{io};
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
    let stream = StreamReader::new(body);

    let drive = Drive::default();
     drive
        .upload(path, stream)
        .await
        .map_err(|err| MiboxError(StatusCode::INTERNAL_SERVER_ERROR, err))
        .map(|_| Ok(StatusCode::CREATED))?
}

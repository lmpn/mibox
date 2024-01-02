use axum::{body::Body, extract::Query, response::IntoResponse};
use hyper::{header, StatusCode};
use tokio_util::io::ReaderStream;
use crate::core::drive::Drive;
use crate::axum_server::error::MiboxError;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn download_service_handler(
    Query(QueryParams { path }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let drive = Drive::default();
    let file = drive.download(path).await.map_err(|err| {
        MiboxError(StatusCode::INTERNAL_SERVER_ERROR, err)
    })?;
    let headers = [(header::CONTENT_TYPE, file.content_type().to_owned())];
    let stream = ReaderStream::from(file);
    let body = Body::from_stream(stream);
    Ok((StatusCode::OK, headers, body))
}

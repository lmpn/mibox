use std::io;

use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use axum::{
    debug_handler,
    extract::{Multipart, Query, State},
    http::StatusCode,
};
use axum_extra::extract::WithRejection;
use futures::TryStreamExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameters {
    path: String,
}

#[tracing::instrument(name = "File upload", skip(application, multipart))]
#[debug_handler]
pub async fn upload_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<Parameters>, MiboxError>,
    mut multipart: Multipart,
) -> Result<StatusCode, MiboxError> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };
        let stream = field.map_err(|e| io::Error::new(io::ErrorKind::Other, e));
        let mut reader = tokio_util::io::StreamReader::new(stream);
        let path = application.drive.join(params.path.clone()).join(file_name);
        let file = tokio::fs::File::create(path.clone())
            .await
            .context(format!("error creating file {:?}", path))?;
        let mut writer = tokio::io::BufWriter::new(file);
        tokio::io::copy(&mut reader, &mut writer)
            .await
            .context("error uploading file")?;
    }
    Ok(StatusCode::OK)
}

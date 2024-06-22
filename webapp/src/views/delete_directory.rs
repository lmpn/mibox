use askama_axum::IntoResponse;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
};
use axum_extra::extract::WithRejection;
use serde::Deserialize;

use crate::{application::Application, error::MiboxError};

use super::boxpath::BoxPath;

#[derive(Debug, Deserialize)]
pub struct RemoveDirectoryParameters {
    path: BoxPath,
}

#[tracing::instrument(name = "Remove directory", skip(application))]
#[debug_handler]
pub async fn delete_directory(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<RemoveDirectoryParameters>, MiboxError>,
) -> impl IntoResponse {
    let path = application.drive.join(params.path);
    let mut h = HeaderMap::new();
    h.insert(CONTENT_TYPE, "text/html".parse().unwrap());
    match tokio::fs::remove_dir_all(path.clone()).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, h, "Something went wrong").into_response()
        }
    }
}

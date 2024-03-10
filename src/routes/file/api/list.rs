use anyhow::Context;
use axum::{extract::Query, http::StatusCode, response::IntoResponse};

use crate::{domain::drive::Drive, error::MiboxError};

#[derive(serde::Deserialize)]
pub struct ListQueryParams {
    count: usize,
    offset: usize,
}

#[derive(serde::Serialize)]
pub struct Page {
    page: Vec<String>,
}

impl From<Vec<String>> for Page {
    fn from(page: Vec<String>) -> Self {
        Self { page }
    }
}

#[tracing::instrument(name = "List file", skip(), fields())]
pub async fn list(
    Query(ListQueryParams { count, offset }): Query<ListQueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let list = Drive::new("tmp".to_string())
        .list(count, offset)
        .await
        .context("failed to list file")?;
    let headers = [(axum::http::header::CONTENT_TYPE, "application/json")];

    Ok((
        StatusCode::OK,
        headers,
        serde_json::json!({"result" : list}).to_string(),
    ))
}

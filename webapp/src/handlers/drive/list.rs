use crate::{application::Application, error::MiboxError};
use anyhow::{anyhow, Context};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::{header::ACCEPT, HeaderMap},
    response::IntoResponse,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ListParameters {
    path: String,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct DriveView {
    pub path: String,
    pub is_directory: bool,
}

#[tracing::instrument(name = "Drive listing", skip(application, headers))]
#[debug_handler]
pub async fn list_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, MiboxError> {
    let entries = Drive::new(application.drive)
        .entries(params.path)
        .await
        .context("list")?;

    let view = entries
        .iter()
        .map(|elem| {
            let path = match elem.name() {
                Some(path) => path,
                None => return None,
            };

            Some(DriveView {
                is_directory: elem.is_directory(),
                path,
            })
        })
        .filter(Option::is_some)
        .map(Option::unwrap)
        .collect::<Vec<DriveView>>();

    let accept_header = headers.get(ACCEPT).context("no accept header")?;
    let accept_header = accept_header.to_str().context("invalid accept header")?;
    if accept_header.contains("*/*") || accept_header.contains("application/json") {
        let j = json!({
            "result" : view
        });
        let j = serde_json::to_value(j).context("error serializing response")?;
        return Ok(axum::Json(j));
    }
    return Err(MiboxError::UnexpectedError(anyhow!(
        "invalid accept header value"
    )));
}

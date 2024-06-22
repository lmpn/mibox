use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use askama::Template;
use axum::{
    debug_handler,
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::{Deserialize, Serialize};

use super::boxpath::BoxPath;

#[derive(Debug, Deserialize)]
pub struct ListParameters {
    path: BoxPath,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct EntryView {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

#[derive(Template)]
#[template(path = "directory.partial.html")]
struct DirectoryPartialTemplate {
    previous: String,
    current: String,
    items: Vec<EntryView>,
}

#[tracing::instrument(name = "Directory html listing", skip(application))]
#[debug_handler]
pub async fn directory(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
) -> Result<impl IntoResponse, MiboxError> {
    let items = Drive::new(application.drive)
        .entries(&params.path)
        .await
        .context("error listing directory")?
        .iter()
        .map(|elem| EntryView {
            is_directory: elem.is_directory(),
            path: format!("{}/{}", params.path.full_name(), elem.file_name()),
            name: elem.file_name(),
        })
        .collect::<Vec<EntryView>>();

    let current = format!("{}/{}", params.path.base(), params.path.name());
    let previous = params.path.base().to_string();
    Ok(DirectoryPartialTemplate {
        previous,
        current,
        items,
    })
}

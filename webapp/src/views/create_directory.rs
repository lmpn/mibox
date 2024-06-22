use crate::{application::Application, error::MiboxError};
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    debug_handler,
    extract::{Query, State},
    response::Redirect,
    Form,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::Deserialize;

use super::boxpath::BoxPath;

#[derive(Debug, Deserialize)]
pub struct CreateDirectoryParameters {
    path: BoxPath,
}

#[derive(Template)]
#[template(path = "create.directory.form.html")]
struct CreateDirectoryFormTemplate {
    current: String,
    directory_name: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateDirectoryForm {
    base: BoxPath,
    directory_name: String,
}

#[tracing::instrument(name = "Create directory form")]
#[debug_handler]
pub async fn create_directory_form(
    WithRejection(Query(params), _): WithRejection<Query<CreateDirectoryParameters>, MiboxError>,
) -> impl IntoResponse {
    CreateDirectoryFormTemplate {
        current: format!("{}/{}", params.path.base(), params.path.name()),
        directory_name: "".to_string(),
    }
}

#[tracing::instrument(name = "Create directory", skip(application))]
#[debug_handler]
pub async fn create_directory(
    State(application): State<Application>,
    Form(form): Form<CreateDirectoryForm>,
) -> impl IntoResponse {
    match Drive::new(application.drive)
        .create_directory(form.base.as_ref().join(&form.directory_name))
        .await
    {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => {
            tracing::error!("{e}");
            CreateDirectoryFormTemplate {
                current: form.base.base().to_string(),
                directory_name: form.directory_name,
            }
            .into_response()
        }
    }
}

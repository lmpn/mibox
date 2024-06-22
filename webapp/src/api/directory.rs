use crate::{application::Application, error::MiboxError};
use anyhow::Context;
use askama::Template;
use axum::{
    debug_handler,
    extract::{Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::WithRejection;
use drive::Drive;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CreateDirParameters {
    path: String,
}

#[tracing::instrument(name = "Create directory", skip(application))]
#[debug_handler]
pub async fn create_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<CreateDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    Drive::new(application.drive)
        .create_directory(params.path)
        .await
        .context("create")?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Template)]
#[template(path = "create.directory.form.html")]
struct CreateDirectoryFormTemplate {
    real_base: String,
    base: String,
    title: String,
}

#[tracing::instrument(name = "Create directory form")]
#[debug_handler]
pub async fn create_dir_form(
    WithRejection(Query(params), _): WithRejection<Query<CreateDirParameters>, MiboxError>,
) -> impl IntoResponse {
    let real_base = params.path.clone();
    let base = if params.path.is_empty() {
        "root".to_string()
    } else {
        params.path
    };
    CreateDirectoryFormTemplate {
        base,
        real_base,
        title: "Create directory".to_string(),
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateDirForm {
    base: String,
    directory_name: String,
}

#[tracing::instrument(name = "Create directory", skip(application))]
#[debug_handler]
pub async fn create_dir(
    State(application): State<Application>,
    Form(form): Form<CreateDirForm>,
) -> impl IntoResponse {
    let path = form.base.strip_prefix("root").unwrap_or(&form.base);
    let path = path.strip_prefix("/").unwrap_or(&path);
    let path = if path == "" {
        form.directory_name
    } else {
        format!("{}/{}", path, form.directory_name)
    };
    match Drive::new(application.drive).create_directory(path).await {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => {
            tracing::error!("{e}");
            let real_base = form.base.clone();
            let base = if form.base.is_empty() {
                "root".to_string()
            } else {
                form.base
            };
            //show error
            CreateDirectoryFormTemplate {
                base,
                real_base,
                title: "Create directory".to_string(),
            }
            .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListParameters {
    path: String,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct EntryView {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct DirectoryView {
    pub results: Vec<EntryView>,
}

#[derive(Template)]
#[template(path = "directory.partial.html")]
struct DirectoryPartialTemplate {
    previous: String,
    base: String,
    items: Vec<EntryView>,
}

#[tracing::instrument(name = "Directory html listing", skip(application))]
#[debug_handler]
pub async fn html_list_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
) -> Result<impl IntoResponse, MiboxError> {
    let original_path = params.path.clone();
    let path = params.path.strip_prefix("root").unwrap_or(&params.path);
    let path = path.strip_prefix("/").unwrap_or(&path);

    let entries = Drive::new(application.drive)
        .entries(&path)
        .await
        .context("error listing directory")?;

    let view = entries
        .iter()
        .map(|elem| {
            let name = match elem.name() {
                Some(name) => name,
                None => return None,
            };
            let path = format!("{original_path}/{name}");
            Some(EntryView {
                is_directory: elem.is_directory(),
                path,
                name,
            })
        })
        .filter(Option::is_some)
        .flatten()
        .collect::<Vec<EntryView>>();

    let base = path.to_string();
    let previous = original_path.split("/").collect::<Vec<_>>();
    let previous = if previous.len() <= 1 {
        "root".to_string()
    } else {
        previous[0..previous.len() - 1].join("/")
    };
    Ok(DirectoryPartialTemplate {
        previous,
        base,
        items: view,
    })
}

#[tracing::instrument(name = "Directory listing", skip(application))]
#[debug_handler]
pub async fn list_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<ListParameters>, MiboxError>,
) -> Result<impl IntoResponse, MiboxError> {
    let entries = Drive::new(application.drive)
        .entries(&params.path)
        .await
        .context("error listing directory")?;

    let view = entries
        .iter()
        .map(|elem| {
            let name = match elem.name() {
                Some(path) => path,
                None => return None,
            };

            let path = if params.path == "" {
                name.clone()
            } else {
                format!("{}/{name}", params.path)
            };

            Some(EntryView {
                is_directory: elem.is_directory(),
                path,
                name,
            })
        })
        .filter(Option::is_some)
        .flatten()
        .collect::<Vec<EntryView>>();
    let view = DirectoryView { results: view };
    let raw = json!(view);
    let json = serde_json::to_value(raw).context("error serializing response")?;
    Ok(axum::Json(json))
}

#[derive(Debug, Deserialize)]
pub struct RemoveDirParameters {
    path: String,
}

#[tracing::instrument(name = "Remove directory", skip(application))]
#[debug_handler]
pub async fn html_remove_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<RemoveDirParameters>, MiboxError>,
) -> impl IntoResponse {
    let path = params.path.strip_prefix("root").unwrap_or(&params.path);
    let path = path.strip_prefix("/").unwrap_or(&path);
    let path = application.drive.join(path);
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
#[tracing::instrument(name = "Remove directory", skip(application))]
#[debug_handler]
pub async fn remove_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<RemoveDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    let path = application.drive.join(params.path.clone());
    tokio::fs::remove_dir_all(path.clone())
        .await
        .context(format!("error removing directory {:?}", path))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct UpdateDirParameters {
    from: String,
    to: String,
}

#[tracing::instrument(name = "Update directory", skip(application))]
#[debug_handler]
pub async fn update_dir_service_handler(
    State(application): State<Application>,
    WithRejection(Query(params), _): WithRejection<Query<UpdateDirParameters>, MiboxError>,
) -> Result<StatusCode, MiboxError> {
    Drive::new(application.drive)
        .rename_directory(params.from, params.to)
        .await
        .context("rename")?;

    Ok(StatusCode::NO_CONTENT)
}

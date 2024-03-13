use anyhow::Context;
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use itertools::Itertools;

use crate::{domain::drive::Drive, startup::AppState};
#[derive(Template)]
#[template(path = "files.html")]
struct FilesTemplate {
    files: Vec<(usize, String)>,
    count: usize,
    offset: usize,
}

#[derive(serde::Deserialize)]
pub struct ListQueryParams {
    count: usize,
    offset: usize,
}
pub async fn files(
    Query(ListQueryParams { count, offset }): Query<ListQueryParams>,
    State(state): State<AppState>,
) -> Html<String> {
    let files = if let Ok(files) = Drive::new("tmp".to_string())
        .list(count, offset)
        .await
        .context("failed to list file")
    {
        files
            .iter()
            .enumerate()
            .map(|(i, e)| (i + count + offset - 10, e.to_owned()))
            .collect_vec()
    } else {
        vec![]
    };
    let files = FilesTemplate {
        files,
        count,
        offset: count + offset,
    };
    Html(files.render().unwrap())
}

use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};

use crate::startup::AppState;
#[derive(Template)]
#[template(path = "files.html")]
struct FilesTemplate {}

#[derive(serde::Deserialize)]
pub struct ListQueryParams {
    count: usize,
    offset: usize,
}
pub async fn files(
    Query(ListQueryParams { count, offset }): Query<ListQueryParams>,
    State(state): State<AppState>,
) -> Html<String> {
    let files = FilesTemplate {};
    let _c = count;
    let _o = offset;
    Html(files.render().unwrap())
}

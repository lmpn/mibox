use askama::Template;
use axum::{extract::State, response::Html};

use crate::startup::AppState;
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    app_name: &'a str,
}

pub async fn home(State(state): State<AppState>) -> Html<String> {
    let home = HomeTemplate {
        app_name: &state.app_name,
    };

    Html(home.render().unwrap())
}


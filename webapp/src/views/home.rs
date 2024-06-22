use askama::Template;

#[derive(Template)]
#[template(path = "home.page.html")]
pub struct HomeTemplate {}

#[tracing::instrument(name = "Home")]
pub async fn home() -> HomeTemplate {
    HomeTemplate {}
}

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let router = Router::new()
            .route("/:file_name", get(get_handler))
            .route("/:file_name", post(post_handler));
        let listener = TcpListener::bind("127.0.0.1:3000").await?;
        axum::serve(listener, router)
            .await.map_err(Into::into)
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn get_handler() -> &'static str {
    "Get"
}
pub async fn post_handler() -> &'static str {
    "Post"
}

use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};
use tokio::signal;
use tokio::{net::TcpListener};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "example_graceful_shutdown=debug,tower_http=debug,axum=trace".into()
                }),
            )
            .with(tracing_subscriber::fmt::layer().without_time())
            .init();
        let router = Router::new()
            .route("/:file_name", get(get_handler))
            .route("/:file_name", post(post_handler))
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
            ));
        let listener = TcpListener::bind("127.0.0.1:3000").await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(Server::shutdown())
            .await
            .map_err(Into::into)
    }

    async fn shutdown() {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("Expecting CTRL+C");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
           _ = ctrl_c => {},
           _ = terminate => {},
        }
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

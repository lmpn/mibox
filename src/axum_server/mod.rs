use axum::{
    routing::{delete, get, post},
    Router,
};
use handlers::{
    download::download_service_handler, fallback::fallback_service_handler,
    listing::listing_service_handler, removal::removal_service_handler,
    search::search_service_handler, upload::upload_service_handler,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod error;
mod handlers;

pub const DRIVE_DIRECTORY: &str = "/Users/luisneto/Documents/dev/mibox/tmp";

pub struct AxumServer {
    address: String,
    timeout: Duration,
    directory: String,
}

impl Default for AxumServer {
    fn default() -> Self {
        Self::new(
            "127.0.0.1:3000".to_string(),
            Duration::from_secs(10),
            DRIVE_DIRECTORY.to_string(),
        )
    }
}

impl AxumServer {
    pub fn new(address: String, timeout: Duration, directory: String) -> Self {
        Self {
            address,
            timeout,
            directory,
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "mibox=info,tower_http=debug,axum=trace".into()),
            )
            .with(tracing_subscriber::fmt::layer().without_time())
            .init();
        let file_routes = Router::new()
            .route("/download", get(download_service_handler))
            .route("/listing", get(listing_service_handler))
            .route("/search", get(search_service_handler))
            .route("/upload", post(upload_service_handler))
            .route("/remove", delete(removal_service_handler));
        let file_router = Router::new().nest("/files", file_routes);
        let versioned_file_router = Router::new().nest("/v1", file_router);
        let router = Router::new()
            .nest("/api", versioned_file_router)
            .fallback(fallback_service_handler)
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(self.timeout),
            ));
        let listener = TcpListener::bind(&self.address).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(AxumServer::shutdown())
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

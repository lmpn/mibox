use crate::configuration::Settings;
use crate::routes::file::api::{download, list, remove, upload};
use crate::routes::health_check::health_check;
use axum::body::Body;
use axum::extract::Request;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub base_url: String,
}

impl AppState {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

pub struct Server {
    address: SocketAddr,
    base_url: String,
}

impl Server {
    pub fn new(address: SocketAddr, base_url: String) -> Self {
        Self { address, base_url }
    }

    pub async fn with_settings(settings: Settings) -> anyhow::Result<Self> {
        let address = settings
            .application
            .address()
            .parse()
            .expect("failed to parse address");

        Ok(Self {
            address,
            base_url: settings.application.base_url,
        })
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        tracing::info!("listening on {}", self.address);
        let listener = tokio::net::TcpListener::bind(self.address)
            .await
            .map_err(|err| {
                tracing::error!("{}", err);
                err
            })?;
        let app = self.create_router().await?;
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(Server::shutdown())
            .await
            .map_err(|err| anyhow::anyhow!("{}", err));
        Ok(())
    }

    pub async fn create_router(&self) -> anyhow::Result<Router> {
        let state = AppState::new(self.base_url.clone());
        let file_api = Router::new()
            .route("/file", get(download).delete(remove).post(upload))
            .route("/file/list", get(list));
        let layer_logging = TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            let request_id = uuid::Uuid::new_v4();
            tracing::span!(
                tracing::Level::INFO,
                "request",
                method = tracing::field::display(request.method()),
                uri = tracing::field::display(request.uri()),
                version = tracing::field::debug(request.version()),
                request_id = tracing::field::display(request_id),
            )
        });
        let layer_timeout = TimeoutLayer::new(Duration::from_secs(10));
        Ok(Router::new()
            .route("/health_check", get(health_check))
            .nest("/api/v1", file_api)
            .with_state(state)
            .layer(layer_timeout)
            .layer(layer_logging))
    }

    pub fn address(&self) -> SocketAddr {
        self.address
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

use crate::application::Application;
use crate::configuration::Settings;
use crate::handlers::drive::create_dir_service_handler;
use crate::handlers::drive::list_service_handler;
use crate::handlers::drive::remove_dir_service_handler;
use crate::handlers::drive::update_dir_service_handler;
use crate::handlers::fallback_service_handler;
use crate::handlers::file::delete_service_handler;
use crate::handlers::file::download_service_handler;
use crate::handlers::file::upload_service_handler;
use crate::handlers::health_check_service_handler;
use axum::body::Body;
use axum::http::HeaderValue;
use axum::middleware;
use axum::routing::delete;
use axum::routing::post;
use axum::routing::put;
use axum::{extract::Request, middleware::Next, response::Response, routing::get, Router};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::trace::TraceLayer;

pub struct Server {
    address: SocketAddr,
    application: Application,
}

impl Server {
    pub fn new(address: SocketAddr, application: Application) -> Self {
        Self {
            address,
            application,
        }
    }

    pub async fn with_settings(settings: Settings) -> anyhow::Result<Self> {
        let address = settings
            .application
            .address()
            .parse()
            .expect("failed to parse address");

        let application = Application::new(
            settings.application.base_url.clone(),
            settings.application.drive.into(),
        );

        Ok(Self {
            address,
            application,
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
            .with_graceful_shutdown(Self::shutdown())
            .await
            .map_err(|err| anyhow::anyhow!("{}", err));
        Ok(())
    }

    pub async fn create_router(&self) -> anyhow::Result<Router> {
        Ok(Router::new()
            .fallback(fallback_service_handler)
            .route("/v1/file", post(upload_service_handler))
            .route("/v1/file", get(download_service_handler))
            .route("/v1/file", delete(delete_service_handler))
            .route("/v1/drive", get(list_service_handler))
            .route("/v1/drive", put(update_dir_service_handler))
            .route("/v1/drive", post(create_dir_service_handler))
            .route("/v1/drive", delete(remove_dir_service_handler))
            .route("/health_check", get(health_check_service_handler))
            .with_state(self.application.clone())
            .layer(middleware::from_fn(secure_headers))
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                    let request_id = uuid::Uuid::new_v4();
                    tracing::span!(
                        tracing::Level::INFO,
                        "request",
                        method = tracing::field::display(request.method()),
                        uri = tracing::field::display(request.uri()),
                        version = tracing::field::debug(request.version()),
                        request_id = tracing::field::display(request_id),
                    )
                }),
            ))
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

    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

async fn secure_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        axum::http::header::X_FRAME_OPTIONS,
        HeaderValue::from_static("deny"),
    );
    response.headers_mut().insert(
        axum::http::header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );
    response
}
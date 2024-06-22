use crate::{
    application::Application,
    configuration::Settings,
    middleware::{secure_headers_layer, tracing_layer},
    views,
};
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::signal;

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
            //.fallback(fallback_service_handler)
            // .route(
            //     "/file",
            //     get(views::upload_files_form).post(views::upload_files),
            // )
            // .route(
            //     "/file",
            //     get(views::file).delete(views::delete_files),
            // )
            .route(
                "/directory",
                get(views::directory).delete(views::delete_directory),
            )
            .route(
                "/directory/create",
                get(views::create_directory_form).post(views::create_directory),
            )
            .route("/", get(views::home))
            .nest_service("/static", tower_http::services::ServeDir::new("static"))
            .layer(axum::middleware::from_fn(secure_headers_layer))
            .layer(tracing_layer())
            .with_state(self.application.clone()))
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

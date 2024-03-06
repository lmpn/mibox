use crate::configuration::Settings;
use crate::routes::health_check;
use axum::body::Body;
use axum::extract::{FromRef, Request};
use axum::{routing::get, Router};
use axum_extra::extract::cookie::Key;
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub base_url: String,
    pub secret: Key,
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.secret.clone()
    }
}

#[derive(Debug, Clone)]
pub struct HmacSecret(pub Secret<String>);

impl AppState {
    pub fn new(pool: PgPool, base_url: String, secret: HmacSecret) -> Self {
        Self {
            pool,
            base_url,
            secret: Key::from(secret.0.expose_secret().as_bytes()),
        }
    }
}

pub struct Server {
    pool: PgPool,
    address: SocketAddr,
    base_url: String,
    hmac_secret: HmacSecret,
}

impl Server {
    pub fn new(
        pool: PgPool,
        address: SocketAddr,
        base_url: String,
        hmac_secret: HmacSecret,
    ) -> Self {
        Self {
            pool,
            address,
            base_url,
            hmac_secret,
        }
    }

    pub async fn with_settings(settings: Settings) -> anyhow::Result<Self> {
        let pool: PgPool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(settings.database.with_db());
        let address = settings
            .application
            .address()
            .parse()
            .expect("failed to parse address");

        Ok(Self {
            pool,
            address,
            base_url: settings.application.base_url,
            hmac_secret: HmacSecret(settings.application.hmac_secret),
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
            .await
            .map_err(|err| anyhow::anyhow!("{}", err));
        Ok(())
    }

    pub async fn create_router(&self) -> anyhow::Result<Router> {
        let state = AppState::new(
            self.pool.clone(),
            self.base_url.clone(),
            self.hmac_secret.clone(),
        );
        // build our application with a route
        Ok(Router::new()
            .route("/health_check", get(health_check))
            .with_state(state)
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

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

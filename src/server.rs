use std::{io, time::Duration};

use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Query, Request},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::TryStreamExt;
use hyper::{header, StatusCode};
use tokio::signal;
use tokio::{io::BufWriter, net::TcpListener};
use tokio_util::io::{ReaderStream, StreamReader};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::MiboxError;

const DRIVE_DIRECTORY : &str = "/Users/luisneto/Documents/dev/mibox/tmp";

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "mibox=info,tower_http=debug,axum=trace".into()),
            )
            .with(tracing_subscriber::fmt::layer().without_time())
            .init();
        let router = Router::new()
            .route("/", get(get_handler))
            .route("/", post(post_handler))
            .fallback(Server::fallback)
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

    async fn fallback() -> impl IntoResponse {
        (StatusCode::NOT_FOUND, "nothing to see here")
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    path: String,
}

pub async fn get_handler(
    Query(QueryParams { path }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => {
            return Err(MiboxError(
                StatusCode::NOT_FOUND,
                anyhow!("File not found: {}", err),
            ))
        }
    };
    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(mime) => mime,
        None => {
            return Err(MiboxError(
                StatusCode::BAD_REQUEST,
                anyhow!("MIME Type couldn't be determined"),
            ))
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [(header::CONTENT_TYPE, content_type)];

    Ok((headers, body))
}
pub async fn post_handler(
    Query(QueryParams { path }): Query<QueryParams>,
    request: Request,
) -> Result<impl IntoResponse, MiboxError> {
    let body = request
        .into_body()
        .into_data_stream()
        .into_stream()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    let mut stream_reader = StreamReader::new(body);

    let path = std::path::Path::new("").join(path);
    if path.is_file() {
        return Err(MiboxError(StatusCode::BAD_REQUEST, anyhow!("File exists")));
    }
    let file = tokio::fs::File::create(path).await.map_err(|err| {
        tracing::error!("{}", err);
        MiboxError(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("{}", err))
    });
    let mut buffer = BufWriter::new(file?);

    tokio::io::copy(&mut stream_reader, &mut buffer)
        .await
        .map(|_| ())
        .map_err(|err| {
            tracing::error!("{}", err);
            MiboxError(StatusCode::INTERNAL_SERVER_ERROR, anyhow!("{}", err))
        })
}

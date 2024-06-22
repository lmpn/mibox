use axum::{
    body::Body,
    http::{HeaderValue, Request, Response},
    middleware::Next,
};
use tower_http::trace::TraceLayer;

pub fn tracing_layer() -> TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&axum::http::Request<Body>) -> tracing::Span + Clone,
> {
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
    })
}

pub async fn secure_headers_layer(request: Request<Body>, next: Next) -> Response<Body> {
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

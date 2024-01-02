use anyhow::anyhow;
use axum::{extract::Query, response::IntoResponse};
use hyper::StatusCode;

use crate::core::drive::Drive;
use crate::axum_server::error::MiboxError;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    query: String,
    page: usize,
    count: usize,
}

pub async fn search_service_handler(
    Query(QueryParams { query, page, count }): Query<QueryParams>,
) -> Result<impl IntoResponse, MiboxError> {
    if count > 50 || count == 0 {
        return Err(MiboxError(
            StatusCode::BAD_REQUEST,
            anyhow!("count must be between 0 and 50"),
        ));
    }

    let files: String = Drive::default()
        .search(&query, page, count)
        .await
        .map_err(|err| {
            tracing::error!("{}", err);
            MiboxError(
                StatusCode::INTERNAL_SERVER_ERROR,
                anyhow!("error loading files"),
            )
        })?;
    Ok((StatusCode::OK, files))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use tower_http::trace::TraceLayer; // for `call`, `oneshot`, and `ready`

    fn app() -> Router {
        Router::new()
            .route("/", get(search_service_handler))
            // We can still add middleware
            .layer(TraceLayer::new_for_http())
    }

    #[tokio::test]
    async fn when_query_parameter_is_missing_return_bad_request() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?page=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            b"Failed to deserialize query string: missing field `query`",
            &body[..]
        );
    }

    #[tokio::test]
    async fn when_page_parameter_is_missing_return_bad_request() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?query=f")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            b"Failed to deserialize query string: missing field `page`",
            &body[..]
        );
    }

    #[tokio::test]
    async fn when_count_parameter_is_missing_return_bad_request() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?query=f&page=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            b"Failed to deserialize query string: missing field `count`",
            &body[..]
        );
    }

    #[tokio::test]
    async fn when_count_is_out_of_bounds_return_bad_request() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let zero_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/?query=f&page=0&count=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(zero_response.status(), StatusCode::BAD_REQUEST);

        let body = zero_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        assert_eq!(b"Error: count must be between 0 and 50", &body[..]);

        let fifty_response = app
            .oneshot(
                Request::builder()
                    .uri("/?query=f&page=0&count=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(fifty_response.status(), StatusCode::BAD_REQUEST);

        let body = fifty_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        assert_eq!(b"Error: count must be between 0 and 50", &body[..]);
    }

    #[tokio::test]
    async fn when_request_is_valid_return_ok() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let zero_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/?query=fd&page=0&count=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(zero_response.status(), StatusCode::OK);

        let body = zero_response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes();
        println!("{:?}", body);
        assert_eq!(b"fd1,fd10,fd11,fd12,fd14,fd15,fd2,fd3,fd4,fd5", &body[..]);
    }
}

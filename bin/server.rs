use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{Stream, TryStreamExt};
const BASE: &str = "tmp";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let router = Router::new()
        .route("/file", get(download_handler))
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn handler<'a>() -> &'a str {
    "healthy"
}

async fn download_handler(Path(params): Path<String>) -> impl IntoResponse {
    let path = BASE.to_owned() + &params;
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let headers = [
        (header::CONTENT_TYPE, "charset=utf-8".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", params),
        ),
    ];
    Ok((headers, body))
}

async fn upload_handler(Path(params): Path<String>, body: Body) -> impl IntoResponse {
    let path = BASE.to_owned() + &params;
    let file = match tokio::fs::File::create(path).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = body.into_data_stream();
    let reader = tokio_util::io::StreamReader::new();
    let headers = [
        (header::CONTENT_TYPE, "charset=utf-8".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", params),
        ),
    ];
    Ok((headers, body))
}

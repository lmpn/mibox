use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{delete, get, put},
    Router,
};
use futures;
use futures::{future, StreamExt, TryStreamExt};
use std::{
    io::{self, ErrorKind},
    path::PathBuf,
};
use tokio::io::BufWriter;
use tokio_stream::wrappers::ReadDirStream;
const BASE: &str = "tmp";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let router = Router::new()
        .route("/file", get(download_handler))
        .route("/file", delete(delete_handler))
        .route("/file", put(upload_handler))
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn handler<'a>() -> &'a str {
    "healthy"
}

async fn download_handler(Query(params): Query<String>) -> impl IntoResponse {
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

    let stream = body
        .into_data_stream()
        .map_err(|err| io::Error::new(ErrorKind::Other, err));
    let mut reader = tokio_util::io::StreamReader::new(stream);
    let mut writer = BufWriter::new(file);
    if let Err(err) = tokio::io::copy(&mut reader, &mut writer).await {
        return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err)));
    };
    Ok(StatusCode::CREATED)
}

async fn delete_handler(Path(params): Path<String>) -> impl IntoResponse {
    let path = BASE.to_owned() + &params;
    let path = PathBuf::from(path);
    let metadata = match tokio::fs::metadata(path.clone()).await {
        Err(_e) => {
            return Ok::<StatusCode, (StatusCode, String)>(StatusCode::NOT_FOUND).into_response()
        }
        Ok(metadata) => metadata,
    };

    if metadata.is_file() {
        if tokio::fs::remove_file(&path).await.is_ok() {
            return Ok::<StatusCode, (StatusCode, String)>(StatusCode::OK).into_response();
        } else {
            return Err::<StatusCode, (StatusCode, String)>((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Couldn't remove file"),
            ))
            .into_response();
        }
    }

    if metadata.is_dir() {
        if tokio::fs::remove_dir_all(&path.clone()).await.is_ok() {
            return Ok::<StatusCode, (StatusCode, String)>(StatusCode::OK).into_response();
        } else {
            return Err::<StatusCode, (StatusCode, String)>((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Couldn't remove directory"),
            ))
            .into_response();
        }
    }
    return Err::<StatusCode, (StatusCode, String)>((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("File not found"),
    ))
    .into_response();
}

async fn list_handler(Path(params): Path<String>) -> impl IntoResponse {
    let path = BASE.to_owned() + &params;
    let path = PathBuf::from(path);
    let metadata = match tokio::fs::metadata(path.clone()).await {
        Err(_e) => {
            return Ok::<StatusCode, (StatusCode, String)>(StatusCode::NOT_FOUND).into_response()
        }
        Ok(metadata) => metadata,
    };

    if metadata.is_file() {
        return Ok::<StatusCode, (StatusCode, String)>(StatusCode::OK).into_response();
    }

    if metadata.is_dir() {
        let dir = tokio::fs::read_dir(path).await.unwrap();
        let stream = ReadDirStream::new(dir);
        let l = stream
            .filter_map(|e| future::ready(e.ok()))
            .map(|e| e.path().to_str().map_or(None, |e| Some(e.to_string())))
            .filter_map(|e| future::ready(e))
            .fold(String::new(), |mut acc, e| {
                if acc.is_empty() {
                    acc += ",";
                }
                acc += &e;
                future::ready(acc)
            })
            .await;
        return Ok::<(StatusCode, String), (StatusCode, String)>((StatusCode::OK, l))
            .into_response();
        // while let Some(entry) = stream.next().await {
        //     println!("GOT = {:?}", v);
        //     let entry = entry;
        //     let path = entry.path();
        // }
        // let l = stream.filter_map(|e| e.ok());
        // for entry in stream {
        //     if path.is_dir() {
        //         visit_dirs(&path, cb)?;
        //     } else {
        //         cb(&entry);
        //     }
        // }
    }
    return Err::<StatusCode, (StatusCode, String)>((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("File not found"),
    ))
    .into_response();
}

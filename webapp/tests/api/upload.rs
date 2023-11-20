use crate::helpers::spawn_app;

#[tokio::test]
async fn when_query_parameters_are_missing_returns_a_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let addr = format!("{}/v1/file", app.address);
    let response = client
        .post(addr)
        .send()
        .await
        .expect("failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Failed to deserialize query string: missing field `path`",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_multipart_is_missing_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let addr = format!("{}/v1/file?path=/", app.address);
    let response = client
        .post(addr)
        .send()
        .await
        .expect("failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Invalid `boundary` for `multipart/form-data` request",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_path_parameter_is_forbidden_returns_500() {
    let app = spawn_app().await;
    let addr = format!("{}/v1/file?path=/", app.address);
    let response = reqwest_multipart_form(&addr).await.unwrap();
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!("Something went wrong", response.text().await.unwrap());
}

#[tokio::test]
async fn when_request_is_wellformed_returns_200() {
    let app = spawn_app().await;
    let addr = format!("{}/v1/file?path=", app.address);
    let response = reqwest_multipart_form(&addr).await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

async fn reqwest_multipart_form(url: &str) -> anyhow::Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let file = tokio::fs::File::open("Cargo.toml").await?;

    // read file body stream
    let stream = tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
    let file_body = reqwest::Body::wrap_stream(stream);

    //make form part of file
    let some_file = reqwest::multipart::Part::stream(file_body)
        .file_name("gitignore.txt")
        .mime_str("text/plain")?;

    //create the multipart form
    let form = reqwest::multipart::Form::new().part("file", some_file);
    //send request
    let response = client.post(url).multipart(form).send().await?;

    Ok(response)
}

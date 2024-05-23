use crate::helpers::spawn_app;

#[tokio::test]
async fn when_query_parameters_are_missing_returns_a_400() {
    let app = spawn_app().await;
    let address = format!("{}/v1/file", app.address);
    let response = app
        .client
        .upload_files(&address, vec![])
        .await
        .expect("error sending files");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Failed to deserialize query string: missing field `path`",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_multipart_is_missing_returns_400() {
    let app = spawn_app().await;
    let address = format!("{}/v1/file?path=/", app.address);
    let response = app
        .client
        .upload_files(&address, vec![])
        .await
        .expect("error sending files");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Invalid `boundary` for `multipart/form-data` request",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_path_parameter_is_forbidden_returns_500() {
    let app = spawn_app().await;
    let address = format!("{}/v1/file?path=/", app.address);
    let response = app
        .client
        .upload_files(&address, vec![("Cargo.toml", "Cargo.toml")])
        .await
        .expect("error sending files");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!("Something went wrong", response.text().await.unwrap());
}

#[tokio::test]
async fn when_request_is_wellformed_returns_200() {
    let app = spawn_app().await;
    let address = format!("{}/v1/file?path=", app.address);
    let response = app
        .client
        .upload_files(&address, vec![("Cargo.toml", "Cargo.toml")])
        .await
        .expect("error sending files");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

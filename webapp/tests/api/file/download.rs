use crate::helpers::spawn_app;

#[tokio::test]
async fn when_file_does_not_exist_returns_500() {
    let app = spawn_app().await;
    let address = format!("{}/api/v1/file?path=hello.txt", app.address);
    let response = app
        .client
        .download_file(&address)
        .await
        .expect("failed to send request");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn when_query_parameters_are_missing_returns_a_400() {
    let app = spawn_app().await;
    let address = format!("{}/api/v1/file", app.address);
    let response = app
        .client
        .download_file(&address)
        .await
        .expect("failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Failed to deserialize query string: missing field `path`",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_query_path_is_a_directory_return_500() {
    let app = spawn_app().await;
    let address = format!("{}/api/v1/file?path=/", app.address);
    let response = app
        .client
        .download_file(&address)
        .await
        .expect("failed to send request");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!("Something went wrong", response.text().await.unwrap());
}

#[tokio::test]
async fn when_request_is_wellformed_returns_200() {
    let app = spawn_app().await;
    app.client
        .upload_files(&app.address, Some(""), vec![("Cargo.toml", "dummy.txt")])
        .await
        .expect("failed to send request");
    let address = format!("{}/api/v1/file?path=dummy.txt", app.address);
    let response = app
        .client
        .download_file(&address)
        .await
        .expect("failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

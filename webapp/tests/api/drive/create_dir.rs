use webapp::handlers::directory::DriveView;

use crate::helpers::spawn_app;

#[tokio::test]
async fn when_query_parameters_are_missing_returns_a_400() {
    let app = spawn_app().await;
    let response = app.client.create_dir(&app.address, "").await;
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Failed to deserialize query string: missing field `path`",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_path_parameter_is_forbidden_returns_500() {
    let app = spawn_app().await;
    let response = app.client.create_dir(&app.address, "/").await;
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!("Something went wrong", response.text().await.unwrap());
}

#[tokio::test]
async fn when_request_is_wellformed_returns_204() {
    let app = spawn_app().await;
    let dir = crate::helpers::random_name(10);
    let response = app.client.create_dir(&app.address, &dir).await;
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    let response = app.client.list(&app.address, "").await;
    let has_dir = response.contains(&DriveView {
        path: dir,
        is_directory: true,
    });
    assert!(has_dir)
}

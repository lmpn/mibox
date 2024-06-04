use webapp::handlers::drive::DriveView;

use crate::helpers::spawn_app;

#[tokio::test]
async fn when_query_parameters_are_missing_returns_a_400() {
    let app = spawn_app().await;
    let address = format!("{}/v1/drive", app.address);
    let response = app
        .client
        .create_dir(&address)
        .await
        .expect("error creating directory");
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        "Failed to deserialize query string: missing field `path`",
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn when_path_parameter_is_forbidden_returns_500() {
    let app = spawn_app().await;
    let address = format!("{}/v1/drive?path=/", app.address);
    let response = app
        .client
        .create_dir(&address)
        .await
        .expect("error sending files");
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
    let address = format!("{}/v1/drive?path={dir}", app.address);
    let response = app
        .client
        .create_dir(&address)
        .await
        .expect("error creating directory");
    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    let address = format!("{}/v1/drive?path=", app.address);
    let response = app
        .client
        .list(&address)
        .await
        .expect("error fetching files")
        .text()
        .await
        .map(|r| serde_json::from_str::<Vec<DriveView>>(&r))
        .unwrap()
        .unwrap();
    println!("{response:?}");
    let has_dir = response.contains(&DriveView {
        path: app.drive_base + "/" + &dir,
        is_directory: true,
    });
    assert!(has_dir)
}

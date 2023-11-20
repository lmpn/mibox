use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_succeeds() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let addr = format!("{}/health_check", app.address);
    let response = client
        .get(addr)
        .send()
        .await
        .expect("failed to send request");
    assert_eq!(response.content_length(), Some(0));
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

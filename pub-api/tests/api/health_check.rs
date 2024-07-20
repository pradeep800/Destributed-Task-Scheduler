use crate::test_helper::spawn;

#[tokio::test]

async fn health_check() {
    let app = spawn().await;
    let client = reqwest::Client::new();
    println!("{:?}", app);
    let body = client
        .get(format!("{}/health-check", app.address))
        .send()
        .await
        .unwrap();
    assert_eq!(body.status(), 200);
}

use crate::test_helpers::spawn;

#[tokio::test]
async fn health_check() {
    let app = spawn().await;
    let res = reqwest::get(format!("{}/health-check", app.address))
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn successful_upload_status() {
    let app = spawn().await;
    let new_task = Task {
        id: 1,
        schedule_at: Utc::now() + Duration::minutes(1),
        picked_at_by_workers: Vec::new(),
        picked_at_by_producers: Vec::new(),
        successful_at: None,
        failed_ats: Vec::new(),
        failed_reasons: Vec::new(),
        total_retry: 3,
        current_retry: 0,
        file_uploaded: true,
        is_producible: true,
        tracing_id: uuid::Uuid::new_v4().to_string(),
    };
    let _inserted_task = new_task.create_task_in_db(&app.db_pool).await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/file/status", app.address))
        .json(&json!({"id":1}))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let json_response = response.json::<UploadStatusResponse>().await.unwrap();
    assert_eq!(json_response.status, "UPLOADED")
}

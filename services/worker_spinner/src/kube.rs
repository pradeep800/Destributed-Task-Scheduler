use anyhow::Result;
use k8s_openapi::api::batch::v1::Job;
use kube::{
    api::{Api, PostParams},
    Client,
};
use tracing::info;
pub async fn create_job(
    signed_url: String,
    jwt: String,
    tracing_id: String,
    host_id: &str,
) -> Result<()> {
    let client = Client::try_default().await?;
    let random_str = uuid::Uuid::new_v4().to_string();
    let name = format!("worker-main-{}", random_str);

    let job: Job = serde_json::from_value(serde_json::json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": name,
        },
        "spec": {
            "backoffLimit": 0,
            "activeDeadlineSeconds": 1200,
            "template": {
                "metadata": {
                    "name": name
                },
                "spec": {
                    "containers": [{
                        "name": "main-container",
                        "image": "pradeep800/worker:latest",
                        "imagePullPolicy": "Always",
                        "env": [
                            {
                                "name": "signed_url",
                                "value": signed_url
                            },
                            {
                                "name": "tracing_id",
                                "value": tracing_id
                            },
                            {
                                "name": "jwt",
                                "value": jwt
                            },
                            {
                                "name": "host_id",
                                "value": host_id
                            }
                        ]
                    }],
                    "restartPolicy": "Never"
                }
            }
        }
    }))?;

    let jobs: Api<Job> = Api::default_namespaced(client);
    let job = jobs.create(&PostParams::default(), &job).await?;
    info!("Job created: {:?}", job.metadata.name);
    Ok(())
}

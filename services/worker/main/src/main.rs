use common::tracing::{get_subscriber, init_subscriber};
use core::panic;
use reqwest::header::{self};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::{
    process::Command,
    select,
    time::{interval, timeout},
};
use tracing::{error, info_span};

const HEART_BEAT_INTERVAL_IN_SECOND: i32 = 5;
const GRACEFUL_SHUTDOWN_TIMEOUT_IN_SECOND: i32 = 20 * 60;
const URL: &str = "http://status-check-svc:80";

struct ChannelBody {
    pub status: String,
    pub body: Option<String>,
}

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("worker".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let mut jwt: Option<String> = None;
    let mut tracing_id: Option<String> = None;
    let mut i = 0;
    if let Ok(lines) = read_lines("/shared/worker.txt") {
        for line in lines.flatten() {
            if i == 0 {
                jwt = Some(line);
            } else if i == 1 {
                tracing_id = Some(line);
            } else {
                break;
            }
            i += 1;
        }
    }
    let tracing_id = tracing_id.unwrap();
    let info_span = info_span!("worker info span", tracing_id = tracing_id);
    let _ = info_span.enter();
    let jwt: Arc<String> = Arc::new(jwt.unwrap());
    let heartbeat_interval = Duration::from_secs(HEART_BEAT_INTERVAL_IN_SECOND as u64);
    let graceful_shutdown_timeout = Duration::from_secs(GRACEFUL_SHUTDOWN_TIMEOUT_IN_SECOND as u64);

    let (tx, mut rx) = mpsc::channel(1);
    let tx_clone = tx.clone();

    let task_with_timeout = tokio::spawn(async move {
        match timeout(graceful_shutdown_timeout, Command::new("/task").status()).await {
            Ok(result) => match result {
                Ok(status) => {
                    if status.success() {
                        if let Err(_err) = tx
                            .send(ChannelBody {
                                status: "SUCCESS".to_string(),
                                body: None,
                            })
                            .await
                        {
                            error_with_panic("Can't send message channel").await;
                        }
                    } else {
                        if let Err(_err) = tx
                            .send(ChannelBody {
                                status: "FAILED".to_string(),
                                body: Some(format!(
                                    "Task status code is {}",
                                    status.code().map_or("not 0".to_string(), |s| s.to_string())
                                )),
                            })
                            .await
                        {
                            error_with_panic("Can't send message to channel").await;
                        }
                    }
                }
                Err(_) => {
                    if let Err(_err) = tx
                        .send(ChannelBody {
                            status: "FAILED".to_string(),
                            body: Some(format!("Task status code is {}", "not 0")),
                        })
                        .await
                    {
                        error_with_panic("Can't send message to channel").await;
                    }
                }
            },
            Err(_) => {
                if let Err(_err) = tx
                    .send(ChannelBody {
                        status: "FAILED".to_string(),
                        body: Some(format!("Task status code is {}", "not 0")),
                    })
                    .await
                {
                    error_with_panic("Can't send message to channel").await;
                }
            }
        };
    });

    let jsonwebtoken = Arc::clone(&jwt);
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = interval(heartbeat_interval);
        let mut i = 0;
        loop {
            match send_heartbeat(&jsonwebtoken).await {
                Ok(_) => {
                    i = 0;
                }
                Err(_) => {
                    i += 1;
                    if i == 6 {
                        if let Err(_err) = tx_clone
                            .send(ChannelBody {
                                status: "FAILED".to_string(),
                                body: Some(format!("Task status code is {}", "not 0")),
                            })
                            .await
                        {
                            error_with_panic("Can't send message to channel").await;
                        }
                    }
                }
            }
            interval.tick().await;
        }
    });
    select! {
        _ = task_with_timeout => {},
        _ = heartbeat_task => {},
        task_body = rx.recv() => {
            send_completion_status_check(&task_body, &Arc::clone(&jwt)).await;
        }
    }
}

async fn send_completion_status_check(task_body: &Option<ChannelBody>, jwt: &Arc<String>) {
    match task_body {
        Some(tb) => {
            send_status(tb, jwt).await;
        }
        None => {
            error_with_panic("Channel didn't send value").await;
        }
    }
}

async fn send_heartbeat(jwt: &Arc<String>) -> Result<(), reqwest::Error> {
    let jwt = format!("{}", jwt);
    let client = reqwest::Client::new();
    let _ = client
        .get(format!("{}/worker/heart-beat", URL))
        .header(header::AUTHORIZATION, jwt)
        .send()
        .await?;
    Ok(())
}

async fn send_status(cb: &ChannelBody, jwt: &Arc<String>) {
    let jsonwebtoken = format!("{}", jwt);
    let client = reqwest::Client::new();
    if cb.status == "SUCCESS" {
        let mut i: u32 = 0;
        let two: u32 = 2;
        loop {
            let res = client
                .post(format!("{}/worker/update-status", URL))
                .header(header::AUTHORIZATION, &jsonwebtoken)
                .json(&serde_json::json!({"status":"SUCCESS"}))
                .send()
                .await
                .unwrap();
            if res.status() != 200 {
                if i == 3 {
                    error_with_panic("Can't able to send status in 3 try").await;
                    break;
                }
                let time = two.pow(i);
                sleep(Duration::from_secs(time as u64)).await;

                i += 1;
                continue;
            }
            break;
        }
    } else {
        client
            .post(format!("{}/worker/update-status", URL))
            .json(&serde_json::json!({"status":"FAILED","failed_reason":cb.status}))
            .header(header::AUTHORIZATION, &jsonwebtoken)
            .send()
            .await
            .unwrap();
    }
}

async fn error_with_panic(message: &str) {
    error!(message);
    sleep(Duration::from_secs(2)).await;
    panic!("{}", message);
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

use core::panic;
use std::time::Duration;
use tokio::{
    process::Command,
    select,
    sync::oneshot,
    time::{interval, timeout},
};
const HEART_BEAT_INTERVAL_IN_SECOND: i32 = 5;
const GRACEFUL_SHUTDOWN_TIMEOUT_IN_SECOND: i32 = 20 * 60;
struct ChannelBody {
    pub status: String,
    pub body: Option<String>,
}
#[tokio::main]
async fn main() {
    let heartbeat_interval = Duration::from_secs(HEART_BEAT_INTERVAL_IN_SECOND as u64);
    let graceful_shutdown_timeout = Duration::from_secs(GRACEFUL_SHUTDOWN_TIMEOUT_IN_SECOND as u64);
    let (tx, rx) = oneshot::channel();
    let task_with_timeout = tokio::spawn(async move {
        match timeout(graceful_shutdown_timeout, Command::new("/task").status()).await {
            Ok(result) => match result {
                Ok(status) => {
                    if status.success() {
                        if let Err(err) = tx.send(ChannelBody {
                            status: "SUCCESS".to_string(),
                            body: None,
                        }) {
                            print!("Can't send message to channel");
                        }
                    } else {
                        if let Err(err) = tx.send(ChannelBody {
                            status: "FAILED".to_string(),
                            body: Some(format!(
                                "Task status code is {}",
                                status.code().map_or("not 0".to_string(), |s| s.to_string())
                            )),
                        }) {
                            print!("Can't send message to channel");
                        }
                    }
                }
                Err(_) => {}
            },
            Err(_) => {}
        };
    });
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = interval(heartbeat_interval);
        loop {
            interval.tick().await;
            send_heartbeat().await;
        }
    });
    select! {
        _ = task_with_timeout => {},
        _ = heartbeat_task => {},
        task_body=rx=>{
            // send response to coordinator about status
        }
    }
}

async fn send_heartbeat() {
    println!("Sending heartbeat to coordinator");
}

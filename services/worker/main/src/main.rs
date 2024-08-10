use core::panic;
use core::panicking::panic;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Duration;
use tokio::{
    process::Command,
    select,
    sync::oneshot,
    time::{interval, timeout},
};
use tracing::error;
use tracing::error;
const HEART_BEAT_INTERVAL_IN_SECOND: i32 = 5;
const GRACEFUL_SHUTDOWN_TIMEOUT_IN_SECOND: i32 = 20 * 60;
struct ChannelBody {
    pub status: String,
    pub body: Option<String>,
}
#[tokio::main]
async fn main() {
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
                            error!("Can't send message to one shot");
                            panic!("Can't send message to one shot");
                        }
                    } else {
                        if let Err(err) = tx.send(ChannelBody {
                            status: "FAILED".to_string(),
                            body: Some(format!(
                                "Task status code is {}",
                                status.code().map_or("not 0".to_string(), |s| s.to_string())
                            )),
                        }) {}
                    }
                }
                Err(_) => {
                    error!("Can't send message to one shot");
                    panic!("Can't send message to one shot");
                }
            },
            Err(_) => {
                error!("Can't send message to one shot");
                panic!("Can't send message to one shot");
            }
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
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

CREATE TABLE health_check_entries (
    task_id INTEGER NOT NULL,
    last_time_health_check TIMESTAMPTZ NOT NULL,
    worker_finished BOOLEAN NOT NULL DEFAULT FALSE,
    pod_name VARCHAR(255) NOT NULL,
    PRIMARY KEY (task_id, pod_name)
);


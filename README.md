# Distributed Task Scheduler
The objective of this project is to create a task scheduler capable of execute static binaries from any compiled language at specified times.

## Architecture Diagram
![Architecture Diagram](images/light.png#gh-light-mode-only)
![Architecture Diagram](images/dark.png#gh-dark-mode-only)

## Architecture Explanation
First, our request will go to a `Public API`. The API will add task detail entry to our `Task Database`. Then, our `Task Producer` will take these entries and add them to an `SQS Queue`. After that, a worker will pick up tasks from the SQS queue and execute them. It will also send health checks to a `Status Check Service` in every 5 seconds. The `Status Check Service` will update the health check time in a `Health Check Database`. When the worker finishes a task, it will send a request to the `Status Check Service`, which will then update the `successful_at` or `failed_ats` with `failed_reasons` in `Task Database`.

### Things to consider before using this service 
1. Your given binary should be idempotent.
2. It will start executing code under 30 seconds. 
3. Maximum task execution time is 20 minute.
4. Retry will happen under 30 seconds(after task fail).

## Component Explanation
### Tasks Database
This database for storing information about tasks
#### Schema

| Attribute             | Keytype                    |
| --------------------- | -------------------------- |
| id                    | Integer (PK)               |
| schedule_at           | timestamptz|null          |
| picked_at_by_producers| timestamptz[]              |
| picked_at_by_workers  | timestamptz[]              |
| successful_at         | timestamptz                |
| failed_ats            | timestamptz[]                |
| failed_reasons        | text[]                       |
| total_retry           | 0-3 (smallint)             |
| current_retry         | 0.3 (smallint) [default 0] |
| tracing_id            | varchar(256)               |
| file_uploaded         | bool [default false]       |
| is_producible         | bool [default true]        |

#### Index
**schedule_at** : because our `Producer` will use `schedule_at` in where clause
### Public API
These APIs will perform 4 functions:

- **Create task (`task/create`):** For creating tasks.
- **Check status (`task/status`):** For checking the status.
- **Create sign URL (`signurl/create`):** Generates a presigned URL for a specific task.
- **Check file posted (`file/status`):** Checks if the file has been posted yet.
 
### producer
basically producer will get data from `Task Database` and publish them into `SQS`
**Working of Producer**
- Lock the database get first 20 entry from `Task Database`
- Publish these entry to `SQS` 
- Add new entry in `picked_at_by_producer` in `Task Database` with current timestamp and `is_producible` to `false`.

**SQL Query**
```sql
SELECT *
FROM Tasks
WHERE schedule_at<= NOW() + INTERVAL 30 SECOND
AND is_producable = true  
AND file_uploaded = true 
Limit 20
FOR UPDATE SKIP LOCKED;
```

- `is_producible` is for whenever you have to make something producible you will make it true this can be used in other services we can just change this attribute to `false` to `true` and our producer will start producing it again


### SQS (Simple Queue Service)

It is a simple FIFO queue which basically can be use as message broker
why are we using `SQS`
- simplicity 
- If we didn't use `SQS` worker have to make multiple connection to database and worker node can be in millions (so million connection)   

### Worker Spinner 
here is what `Worker Spinner` container is doing 
- Polling for a task entry from SQS.
- Creating S3 signed url which give access to task file for 20 minute.
- Creating random uuid for host_id.
- Creating a JWT which include task_id, host_id, tracing_id and pod_name. 
- Create new  `Worker` container where we will provide env variable given below
   - host_id
   - tracing_id
   - jwt
   - signed url
- add new entry at `picked_at_by_worker` to the current time.

### Worker
The main container performs the following tasks:
- Get info about worker from environmental variable.
- Download file from S3 and execute the file.
- Keep sending heart beat to `Status Check Service`.
- Sends task success or failure notifications to the `Status Check Service`.

**Why JWT?**

Because our worker node's main container runs our health check and completion logic. It's crucial for us to ensure secure communication since we don't fully trust the worker node. Therefore, we provide tokens to the worker node, restricting in a way that it can only affect only his state.

### Health check database
we are going to use this database for collecting health information

**schema**
health_check_entries table
| name                             | type     |
| -------------------------------- | -------- |
| task_id                          | int (PM)    |
| last_time_health_check           | timestamptz |
| worker_finished                  | bool (false)|
| pod_name                         | varchar(255)|


**Index**
1. (task_id,pod_name) => 
Because in health checker service we are using both of them into where clause

### Status Checker Service
Every worker will send heart beat in every 5 seconds and when worker will finish it job it will send it status to our `Status Checker Service`
Here are 2 things `Status Check Service` does
- Provide API for updating `last_time_health_check` in `Health Check Database`
- Provide API for updating the status of worker in `Task Db`

**Working of `Status Check Service`**

- **POST /health_check:** it will update the value of last_time_health_check to current timestamp.

```sql
INSERT INTO health_check_entries(task_id, last_time_health_check,pod_name)
VALUES ($1, NOW(),$2)
ON CONFLICT (task_id,pod_name)
DO UPDATE SET
last_time_health_check = NOW()
WHERE worker_finished=false 
```

- **POST /update_status:** it will update the status of worker 

If our worker send us we successfully completed the task


```sql
SELECT * from tasks WHERE id= $1  FOR UPDATE

UPDATE health_check_entries
SET worker_finished= true
WHERE task_id = :task_id
AND pod_name= :pod_name
AND worker_finished=false

```


```sql
UPDATE tasks
SET successful_at= NOW()
WHERE id = :task_id;
```

If our worker task got failed
we'll check if total_retry = current_retry

```sql
UPDATE tasks SET
failed_at = array_append(failed_at, now()),
failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
```


If our total_retry > current_retry

```sql

UPDATE tasks
SET is_producible = true,
current_retry = current_retry + 1,
failed_at = array_append(failed_at, now()),
failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
```

### Retry and Failed Updater Service
This service identifies tasks exceeding a 20-second (4 health-check failed) health check interval as dead worker, updating the `task database` with `failed_at` and a `failed_reason`.

This service is only made for something unintended happen for example
- network partition for more than 20 seconds
- our task is taking more than 1 CPU 
- our task is interfering with our heart beat logic

This service worker with these assumption
- Every worker is sending heart beats to our `Status Check Service` in every 5 seconds

**Some ideas for future**
For making sure that the container we think are terminated are terminated. we have to use k3s api and use pod_name to delete those containers forcefully

Sql query for checking task which didn't send health-check for 20-second



```sql
SELECT *
FROM health_check_entries
WHERE last_time_health_check < NOW() - INTERVAL '20 seconds'
AND worker_finished = false
ORDER BY task_id, pod_name
LIMIT 10
FOR UPDATE SKIP LOCKED
```


iterate through every single entry in health_check_entries
and  do this first


```sql
SELECT * from Tasks where id=:hc[i].id FOR UPDATE

UPDATE health_check_entries SET 
worker_finished=true where tasks_id=:task_id AND pod_name=:pod_name
```

-  if total_retry = current_retry


```sql
UPDATE tasks
SET failed_at = NOW(),
failed_at = array_append(failed_at, now()),
failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
```

- if our total_retry > current_retry

```sql
UPDATE tasks
SET is_producible = true,
    current_retry = current_retry + 1,
    failed_at = array_append(failed_at, now()),
    failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
```


### Remove Health Check Database Entries (Remove HS DB Entries)
because we know we are terminating task in 20 minute so even in network failure it should have entry which will be not updated because worker_finished=true (that mean they spwan new worker)

This cron job executes in every 5 minutes to remove obsolete entries from the `Health Check Database` that are no longer needed. 

Assumption this service make
- Our main worker will make sure that our task will be finished in 20 minute

**SQL query**

```sql
DELETE FROM health_check_entries 
WHERE worker_finished= true
AND last_time_health_check >= NOW()- INTERVAl 20 MIN
```


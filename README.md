# Distributed Task Scheduler
The objective of this project is to create a task scheduler capable of executing binaries from any compiled language at specified times.

## Architecture Diagram
![Architecture Diagram](images/light.png#gh-light-mode-only)
![Architecture Diagram](images/dark.png#gh-dark-mode-only)

## Architecture Explanation
First, our request will go to a `Public API`. The API will add task detail entry to our `Task Database`. Then, our `Task Producer` will take these entries and add them to an `SQS Queue`. After that, a worker will pick up tasks from the SQS queue and execute them. It will also send health checks to a `Status Check Service` in every 5 seconds. The `Status Check Service` will update the health check time in a `Health Check Database`. When the worker finishes a task, it will send a request to the `Status Check Service`, which will then update the `successful_at` or `failed_at` with `failed_reason` in `Task Database`.

## Component Explanation
### Tasks Database
This database for storing information about tasks
#### Schema

| Attribute             | Keytype                    |
| --------------------- | -------------------------- |
| id                    | Integer (PK)               |
| schedule_at           | timestamptz\|null          |
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
- **Create sign URL (`signurl/create`):** Generates a sign URL for a specific task.
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

- `is_producible` is for whatever you have to make something producible you will make it true this can be used in other services we can just change this attribute to `false` to `true` and our producer will start producing it again


### SQS (Simple Queue Service)

It is a simple FIFO queue which basically can be use as message broker
why are we using `SQS`
- simplicity 
- If we didn't use `SQS` worker have to make multiple connection to database and worker node can be in millions (so million connection)   

### Worker Service
Worker service consists of three main components:
1. **Share Volume:**
   Share volume is the storage shared between the init container and the main container. It stores:
   - The executable file.
   - `jwt.txt`.

2. **Init Container:**
   here is what init container is doing
   - Polling for a single task entry from SQS.
   - Retrieving the executable file from S3.
   - Storing the retrieved S3 file into the share volume.
   - Creating a JWT and saving it in a file named `jwt.txt`.
   - add new entry at `picked_at_by_worker` to the current time.

3. **Main Container:**
   The main container performs the following tasks:
   - Executes a file providing health checks and completion updates to the `Status Check Service`. It also parses the `jwt.txt` file from the share volume.
   - Runs the executable file from the share volume by code.
   - Sends task success or failure notifications to the `Status Check Service`.


**Why JWT?**

Because our worker node's main container houses our health check and completion logic. It's crucial for us to ensure secure communication since we don't fully trust the worker node. Therefore, we provide tokens to the worker node, restricting its requests solely to itself.

### Health check database
we are going to use this database for collecting health information

**schema**

| name                             | type     |
| -------------------------------- | -------- |
| id                               | int (PM) |
| task_id                          | int      |
| last_time_health_check           | timestamptz |
| task_completed                   | bool     |


**Index**
(last_time_health_check,task_completed) => 
- in the `Remove Health Check Database Entry` operation, we remove entries from the database for tasks that workers have already completed.
- In `Retry and Failed Updater` we are going to use both or key in where clause so I am making in index

### Status Checker Service
Every worker will send status check in every 5 seconds and when worker will finish it job it will send it status to our `Status Checker Service`
Here are 2 things `Status Check Service` does
- Provide API for updating `last_time_health_check` in `Health Check Database`
- Provide API for updating the status of worker in `Task Db`

**Working of `Status Check Service`**

- **POST /health_check:** it will update the value of last_time_health_check to current timestamp.

```sql
INSERT INTO HealthCheckEntries (task_id, last_time_health_check, task_completed)
VALUES ($1, NOW(), true)
ON CONFLICT (task_id)
DO UPDATE SET
  last_time_health_check = NOW(),
  task_completed= true;
```

- **POST /update_status:** it will update the status of worker 

If our worker send us we successfully completed the task

```sql
-- with this query Retry and Failed updater service will not select this
UPDATE your_table_name
SET task_completed= true
WHERE task_id = :task_id;

UPDATE tasks
SET successful_at= NOW()
WHERE id = :task_id;
```

If our worker said our task got failed
we'll check if total_retry = current_retry

```sql
UPDATE tasks
    SET failed_at = NOW(),
        failed_at = array_append(failed_at, now()),
        failed_reason = array_append(failed_reason, :reason)
    WHERE id = :task_id
    AND total_retry = current_retry
    FOR UPDATE
```


If our total_retry > current_retry

```sql
UPDATE tasks
SET is_producible = true,
    current_retry = current_retry + 1,
    failed_at = array_append(failed_at, now()),
    failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
AND current_retry < total_retry;
FOR UPDATE
```

### Retry and Failed Updater Service
This service identifies tasks exceeding a 20-second (4 health-check failed) health check interval as dead worker, updating the `task database` with `failed_at` and a `failed_reason`.

Sql query for checking task which didn't send health-check for 20-second

```sql
SELECT *
FROM health_check 
WHERE last_time_health_check < NOW() - INTERVAL '20 seconds'
AND task_completed= true;
```

If the task has retries remaining
- we'll check if total_retry = current_retry

```sql
-- with this query Retry and Failed updater service will not select this
UPDATE your_table_name
SET task_completed= true
WHERE task_id = :task_id;

UPDATE tasks
    SET failed_at = NOW(),
        failed_at = array_append(failed_at, now()),
        failed_reason = array_append(failed_reason, :reason)
    WHERE id = :task_id
    AND total_retry = current_retry

```
- if our total_retry > current_retry

```sql
UPDATE tasks
SET is_producible = true,
    current_retry = current_retry + 1,
    failed_at = array_append(failed_at, now()),
    failed_reason = array_append(failed_reason, :reason)
WHERE id = :task_id
AND current_retry < total_retry;
```

### Remove Health Check Database Entries (Remove HS DB Entries)

This cron job executes in every 10 minutes to remove obsolete entries from the `Health Check Database` that are no longer needed. 

**SQL query**

```sql
DELETE FROM health_check 
WHERE task_completed= true;
```
### Things to consider before using this service 

1. Your given binary should be idempotent 
2. It will execute our under 1 minute
3. Maximum task execution time is 20 minute
4. Retry will happen under 1 minute

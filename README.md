# Distributed Task Scheduler
The objective of this project is to create a task scheduler capable of executing binaries from any compiled language at specified times.

## Architecture Diagram
![Architecture Diagram](images/light.png#gh-light-mode-only)
![Architecture Diagram](images/dark.png#gh-dark-mode-only)

## Architecture Explanation
First, our request will go to a `Public API`. The API will add task detail entry to our `task database`. Then, our `task producer` will take these entries and add them to an `SQS queue`. After that, a worker will pick up tasks from the SQS queue and execute them. It will also send health checks to a `status check service` every 5 seconds. The status check service will update the health check time in a `health check database`. When the worker finishes a task, it will send a request to the `status check service`, which will then update the `completed_at` and `failed_at` with `failed_reason` in `task database`.

## Component Explanation
### Tasks Database
This database for storing information about tasks
#### Schema

| Attribute             | Keytype                    |
| --------------------- | -------------------------- |
| id                    | Integer (PK)               |
| schedule_at           | timestamptz\|null           |
| picked_at_by_producer | timestamptz[]               |
| picked_at_by_worker   | timestamptz[]               |
| completed_at          | timestamptz                 |
| failed_at             | timestamptz                 |
| failed_reason         | text                       |
| total_retry           | 0-3 (smallint)             |
| current_retry         | 0.3 (smallint) [default 0] |
| file_uploaded         | bool                       |

#### Index
**schedule_at** : Since our producer always checks the schedule promptly to expedite production.

### Public API
These APIs will perform 4 functions:

- **Create task (`task/create`):** For creating tasks.
- **Check status (`task/status`):** For checking the status.
- **Create sign URL (`signurl/create`):** Generates a sign URL for a specific task.
- **Check file posted (`task/fileposted/check`):** Checks if the file has been posted yet.
 
### producer
The producer retrieves task information from the `task database` places them into a queue, and updates the `picked_at_by_producer` array in the `task database` with timestamps.

**SQL Query for Querying Information with Locking:**

```sql
SELECT *
FROM tasks
WHERE schedule_at_in_second >= NOW() 
  AND schedule_at_in_second <= NOW() + INTERVAL 30 SECOND
  AND JSON_LENGTH(picked_at_by_producer) = 0
FOR UPDATE SKIP LOCKED;
```



### Sqs (Simple Queue Service)

It is a simple FIFO queue which basically can be use as message broker
why are we using sqs
- simplicity 
- If we didn't use this worker have to make multiple connection to database worker node can be in millions (so million connection)   

### Worker Service

Worker service consists of three main components:

1. **Share Volume:**
   Share volume is the storage shared between the init container and the main container. It stores:
   - The executable file intended for execution.
   - `jwt.txt`, where the JWT is written by the init container.

2. **Init Container:**
   here is what init container is doing
   - Polling for a single task entry from SQS.
   - Retrieving the executable file from S3.
   - Storing the retrieved S3 file into the share volume.
   - Creating a JWT and saving it in a file named `jwt.txt`.
   - add new entry at `picked_at_by_worker` to the current time.

3. **Main Container:**
   The main container performs the following tasks:
   - Executes a file providing health checks and completion updates to the `status check service`. It also parses the `jwt.txt` file from the share volume.
   - Runs the executable file from the share volume by code.
   - Sends task success or failure notifications to the `status check service`, which logs them in the task database using the provided JWT.

    we should not trust not trust main container because it is running someone else code so basically we will give them access to things which can only change own state 

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
| task_updated                     | bool     |


**Index**
(last_time_health_check,task_updated) => 
- in `remove health check` we are removing the entries 3 minute which are 3 minute late to update their last_time_health
- In `failed updatator` we are going to use both or key in where clause so i am making in index
### Status Checker Service
Every worker pings the status check service every 5 seconds for health checks. When a task is completed or failed, the worker sends its status and this service will update accordingly Here's what the `status checker service` does:
1. Create 2 API endpoints accessible to the main worker:
   - **POST /health_check:** Sends health information with the body `{jwt: string}`.
   - **POST /update_status:** update the status of worker
2. when it send request to health_check update `last_time_health_check` to the current time.
3. When a task completes or fails 
   - update the `health check database` Set `task_updated` to true.
   - Update `completed_at` or `failed_at` in the `task database`.
# Think about what will happen when it failed and retry are there
### Retry and Failed Updater Service
This service identifies tasks exceeding a 30-second health check interval as dead tasks, updating the `task database` with `failed_at` and a `failed_reason`. If the task has retries remaining, it queues it in SQS with `current_retry + 1` and add new `picked_at_by_producer` entry in `task database`.

### Remove Health Check Database Entries (Remove HS DB Entries)
This cron job executes every 3 minutes to remove obsolete entries from the health check database that are no longer needed.


### How i am going to maintain SLA

# Distributed Task Scheduler
The objective of this project is to create a task scheduler capable of executing binaries from any compiled language at specified times.

## Architecture Diagram
![Architecture Diagram](images/light.png#gh-light-mode-only)
![Architecture Diagram](images/dark.png#gh-dark-mode-only)

## Architecture Explaination
First, our request will go to a "public API". The API will add task detail entrie to our "task database". Then, our "task producer" will take these entries and add them to an "SQS queue". After that, a worker will pick up tasks from the SQS queue and execute them. It will also send health checks to a "status check service" every 5 seconds. The status check service will update the health check time in a "health check database". When the worker finishes a task, it will send a request to the “status check service”, which will then update the completed_at in “task database”.

## Component Explaination


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
The producer retrieves task information from the "task database," places them into a queue, and updates the "picked_at_by_producer" array in the "task database" with timestamps.

**SQL Query for Querying Information with Locking:**

```sql
SELECT *
FROM tasks
WHERE schedule_at_in_second >= NOW() 
  AND schedule_at_in_second <= NOW() + INTERVAL 30 SECOND
  AND JSON_LENGTH(picked_at_by_producer) = 0
FOR UPDATE SKIP LOCKED;
```



##### Sqs (Simple Queue Service)

It is a simple FIFO queue which basically can be use as message broker
why are we using sqs
- simplicity 
- If we didn't use this worker have to make multiple connection to database worker node can be in millions (so million connection)
how are we using sqs in our architecture
so we use sqs as message broker between producer and worker


##### Worker Service

Worker service is made up of 3 things 
1. Share volume:- basically share volume is volume share between init container and main container here is what share volumne is storing
	- the file which we want to execute
	- jwt.txt file where jwt is written by init conatiner 
2. Init container  :- which will basically help it do initialize container for running jobs it will do given below things 
	 - it will poll for  one task entry from sqs
	 - it will get executable file from s3
	 - it will store that s3 file into share volume
	 - It will create jwt and put it into file called jwt.txt 
	 - It will change last_time_picked_at_by_worker to current time
	 
Why JWT? 

Because our worker node's main container (houses our health check and completion logic. It's crucial for us to ensure secure communication since we don't fully trust the worker node. Therefore, we provide tokens to the worker node, restricting its requests solely to itself.

3.  Main container :- it will do given below things
      - Run a file which is giving health check and completion update to "status check service"  and also parse file name jwt.txt which is fromshare volumne
      - Running the Executable file from share volume 
      - when our task get failed or successful we will send it to "status check service" which will add them to "task database with given JWT
    we should not trust not trust main container because it is running someone else code so basically we will give them access to things which can only change his state 

##### Health check database
we are going to use this database for collecting health information

**schema**

| name                             | type     |
| -------------------------------- | -------- |
| id                               | int (PM) |
| task_id                          | int      |
| last_time_health_check_in_second | int      |
| task_updated                     | bool     |


**Index**
(last_time_health_check_in_second,task_updated) => 
- in "remove health check" we are removing the entries 3 minute which are 3 minute late to update their last_time_health_in_second
- In "failed updatator" we are going to use both or key in where clause so i am making in index


##### Status Checker Service
Every  worker will keep pining the state check service in every 5 second for health check and when some task got finished worker will send the status of task
here is what status checker service is going to do
1. we will create 2 api endpoint which are rechable by main worker
    - POST health_check : - for sending their information about helath 
     body : {jwt:string}
     - POST update_status : for updating the state of our worker which we can do by updating our failed_at or completed_at in "task database"
1. keep updating their last_time_health_check_in_second to their new time
2. when our task send us complete or failed we'll update the "health check database" and put task_updated as true and in "task database" we will update "completed_at" or failed_at

##### Retry and Failed Updator Service
In this service if last_time_health_check_in_second not under 30 second limit we will consider it as dead task and update our "task database" with failed_at and with reason
and if it have retry left we will not do anything we will just put it in sqs queue and add current_retry +1 

##### Remove Health Check database entries (remove HS DB entries)
This is a cron job which will execute in every 3 minute to remove the entries which are old and not needed in database 


##### How i am going to maintain SLA






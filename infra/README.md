# Before running command you should install these things
1. kubectl
2. minikube
3. helm

# Here is how to set it up

### Set up secrets

first be inside infra folder

```
kubectl apply -f secrets.yaml
```

### Set up tasks db

```
helm install tasks-db . -f values/tasks_db.yaml
```

### set up health_check db

```
helm install tasks-db . -f values/tasks_db.yaml
```

### Do migration in those database

```
kubectl apply -f migration.yaml
```

### Set up public API
 
```
helm install  pub-api . -f values/pub_api.yaml
```

### Set up producer 

```
helm install  producer . -f values/tasks_producer.yaml
```

### Set up worker init and main 

```
```

### Set up status check 

```
helm install  status-check . -f values/status_check.yaml
```

### set up health check remover
```
helm install  health-check-remover . -f values/health_check_remover.yaml
```

### set up retry and failed updater
```
helm install retry-and-failed . -f values/retry_and_failed_updater.yaml
```



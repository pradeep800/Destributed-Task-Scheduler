```
ssh -L 3008:localhost:3008
kubectl port-forward service/grafana-svc 3008:80


ssh -L 9999:localhost:9999 kube
kubectl port-forward service/pub-api-svc 9999:80
```

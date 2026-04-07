# 🚀 Deployment Guide

## Quick Deployment Summary

| Platform | Time | Command | Ideal For |
|----------|------|---------|-----------|
| **Docker Compose** | 30 sec | `docker-compose up -d` | Local dev, testing |
| **Kubernetes** | 2 min | `kubectl apply -f kubernetes/` | Production, cloud |
| **Terraform + AWS** | 5 min | `cd terraform && terraform apply` | AWS infrastructure |

---

## 🐳 Docker Compose (Local Development)

### Setup

```bash
# Check Docker is running
docker ps

# Start 4-broker cluster + monitoring
docker-compose up -d

# Verify
docker ps
```

### What's Included

```yaml
Services:
  └─ 4 FastDataBroker brokers
  └─ Prometheus (metrics)
  └─ Grafana (dashboards)
```

### Access Points

| Service | URL | Purpose |
|---------|-----|---------|
| **Broker 0** | localhost:8080 | Send/receive messages |
| **Broker 1** | localhost:8081 | Send/receive messages |
| **Broker 2** | localhost:8082 | Send/receive messages |
| **Broker 3** | localhost:8083 | Send/receive messages |
| **Prometheus** | localhost:9090 | Metrics |
| **Grafana** | localhost:3000 | Dashboards |

### Testing Locally

```python
from postoffice_sdk import ClusterClient, Producer, Consumer

# Easy localhost test
client = ClusterClient([
    'localhost:8080', 'localhost:8081',
    'localhost:8082', 'localhost:8083'
])

producer = Producer(client)
key = producer.send(b'test', b'hello')

consumer = Consumer(client, 'test-group')
msg = consumer.consume()
print(msg.value)  # Output: b'hello'
```

### Logs

```bash
# View logs
docker-compose logs broker0  # Single broker
docker-compose logs -f       # All services, follow

# Clear logs
docker-compose logs --tail=100 broker0
```

### Stop/Restart

```bash
# Stop gracefully
docker-compose stop

# Restart
docker-compose restart

# Full reset (dangerous - loses data)
docker-compose down
docker-compose up -d
```

---

## ☸️ Kubernetes (Production)

### Prerequisites

```bash
# Install kubectl
brew install kubectl  # macOS
apt-get install kubectl  # Linux
# Download from https://kubernetes.io/docs/tasks/tools/

# Configure cluster access
kubectl config current-context  # Verify you're on right cluster
```

### Deploy to Kubernetes

```bash
# Deploy entire stack
kubectl apply -f kubernetes/

# Verify
kubectl get statefulsets    # Should show fastdatabroker-cluster
kubectl get pods            # Should show 4 running brokers
kubectl get svc             # Should show services

# Check logs
kubectl logs fastdatabroker-cluster-0
kubectl logs fastdatabroker-cluster-1
```

### What's Included

```yaml
StatefulSet:
  └─ 4-broker cluster with persistent volumes
     ├─ Ordered startup/shutdown
     ├─ Stable network identities  
     └─ Automatic failover

Services:
  ├─ Headless (broker-broker communication)
  ├─ ClusterIP (internal access)
  └─ LoadBalancer (external access)

RBAC:
  ├─ ServiceAccount
  ├─ Role (permissions)
  └─ RoleBinding

Monitoring:
  ├─ ServiceMonitor (Prometheus scraping)
  └─ Pod annotations (metrics)
```

### Access Patterns

**From Inside Cluster**:
```python
# Use service DNS
client = ClusterClient([
    'fastdatabroker-cluster-0.fastdatabroker:8080',
    'fastdatabroker-cluster-1.fastdatabroker:8080',
    'fastdatabroker-cluster-2.fastdatabroker:8080',
    'fastdatabroker-cluster-3.fastdatabroker:8080',
])
```

**From Outside Cluster**:
```bash
# Port-forward to test
kubectl port-forward svc/fastdatabroker 8080:8080

# Then access via localhost:8080
```

**Production Load Balancer**:
```bash
# Get external IP (if LoadBalancer type)
kubectl get svc fastdatabroker
# Shows: EXTERNAL-IP: 34.56.78.90

# Use that IP
client = ClusterClient(['34.56.78.90:8080', ...])
```

### Auto-Scaling

Current configuration:
```yaml
replicas: 4  # Fixed at 4 for optimal quorum

# Horizon: Configurable replicas (v2.0)
```

### Persistent Data

```bash
# Check volumes
kubectl get pvc

# Backup data
kubectl exec fastdatabroker-cluster-0 -- tar czf - /data | gzip > broker0-backup.tar.gz

# Restore
kubectl exec -i fastdatabroker-cluster-0 -- tar xzf - /data < broker0-backup.tar.gz
```

### Network Policies

```yaml
# Current: Open to all pods in namespace
# Future: NetworkPolicy restricting traffic
```

### Resource Limits

```yaml
Requests:
  CPU: 500m
  Memory: 512Mi

Limits:
  CPU: 1000m
  Memory: 1024Mi
```

Adjust in `kubernetes/02-statefulset-service.yaml` as needed.

### Updating the Cluster

```bash
# Rolling update to new image version
kubectl set image statefulset/fastdatabroker \
  fastdatabroker=fastdatabroker:v2.0.0

# Watch progress
kubectl rollout status statefulset/fastdatabroker

# Rollback if needed
kubectl rollout undo statefulset/fastdatabroker
```

---

## ☁️ Terraform (AWS Infrastructure)

### Prerequisites

```bash
# Install Terraform
brew install terraform  # macOS
# Or download from https://www.terraform.io/downloads.html

# Configure AWS credentials
aws configure
# Provide AWS Access Key ID and Secret Access Key

# Verify
terraform version
aws sts get-caller-identity
```

### Deploy to AWS

```bash
cd terraform

# Initialize Terraform
terraform init

# Plan (see what will be created)
terraform plan

# Apply (actually create resources)
terraform apply

# Review outputs
terraform output
```

### What's Created

```
AWS Resources:
├─ VPC (Virtual Private Cloud)
│  └─ Public + Private subnets
├─ EC2 Instances (4x t3.large)
│  ├─ Security groups
│  ├─ IAM roles
│  └─ EBS volumes (storage)
├─ Networking
│  ├─ Load balancer
│  ├─ Auto Scaling Group
│  └─ Route53 DNS
└─ Monitoring
   ├─ CloudWatch alarms
   └─ CloudWatch logs
```

### Costs

```
Monthly expenses:
├─ 4x t3.large instances:    ~$120
├─ 4x 100GB gp3 EBS volumes: ~$40  
├─ Load balancer:            ~$20
├─ Data transfer:            ~$200 (depends on usage)
└─ Total estimate:           ~$400/month
```

### Accessing the Cluster

```bash
# Get IP address
terraform output brokers_ip

# SSH into broker
ssh -i fastdatabroker.pem ubuntu@<IP>

# Check health
curl http://<IP>:8080/health

# View logs
tail -f /var/log/fastdatabroker.log
```

### Scaling Up/Down

```bash
# Change instance count
terraform apply -var="broker_count=6"

# Change instance type
terraform apply -var="instance_type=t3.xlarge"

# Destroy entire infrastructure (warning!)
terraform destroy
```

### Monitoring with CloudWatch

```bash
# Enable detailed monitoring
aws cloudwatch put-metric-alarm \
  --alarm-name fastdatabroker-latency \
  --metric-name p99_latency \
  --namespace FastDataBroker
```

---

## 📊 Monitoring & Observability

### Prometheus Metrics

**Available Metrics**:
```
fastdatabroker_messages_sent_total    # Total messages sent
fastdatabroker_messages_received_total # Total consumed
fastdatabroker_latency_p99_ms         # P99 latency
fastdatabroker_throughput_msg_sec     # Current throughput
fastdatabroker_broker_alive           # Broker health (0/1)
```

### Grafana Dashboard

```bash
# Access Grafana
# Docker: localhost:3000
# Kubernetes: kubectl port-forward svc/prometheus-grafana 3000:3000

# Default credentials:
# Username: admin
# Password: prom-operator

# Import dashboard:
# 1. Click "+" → "Import"
# 2. Paste dashboard ID: [coming soon]
# 3. Select Prometheus data source
```

### Health Checks

**Broker Health**:
```bash
# HTTP health check
curl http://localhost:8080/health
# Response: {"status": "healthy"}

# Kubernetes liveness probe
# kubectl describe pod fastdatabroker-cluster-0

# Check replica lag
curl http://localhost:8080/metrics | grep replica_lag_ms
```

### Alerting

**Critical Alerts**:
```yaml
- name: BrokerDown
  condition: fastdatabroker_broker_alive == 0
  duration: 30s
  action: PagerDuty

- name: HighLatency
  condition: fastdatabroker_latency_p99_ms > 10
  duration: 5m
  action: Slack + Opsgenie

- name: ReplicaLagging
  condition: replica_lag_ms > 5000
  duration: 1m
  action: Email ops team
```

---

## 🔐 Security

### Authentication (Planned v2.0)

```python
# Future: API tokens
client = ClusterClient(
    bootstrap_servers=[...],
    auth_token="sk_prod_12345..."
)
```

### Current: No Authentication
- Deploy in VPC
- Restrict firewall/security groups
- Use network policies in Kubernetes

### Encryption in Transit (Planned v2.0)

```bash
# Future: TLS support
# export FDB_USE_TLS=true
# export FDB_CERT_PATH=/etc/certs/
```

### Current: Plain HTTP
- Use in trusted networks only
- Put behind TLS load balancer (nginx, HAProxy)

### Data Encryption at Rest

```bash
# Docker: Use encrypted volumes
docker volume create --driver local \
  --opt type=tmpfs ...

# Kubernetes: Use encrypted EBS volumes (AWS)
# Terraform: Configure EBS encryption
```

---

## 🚨 High Availability Setup

### 4-Broker Cluster (Recommended)

```
Topology:
  └─ 4 brokers (2 in AZ-a, 2 in AZ-b)
  └─ Can tolerate: 1 broker failure
  └─ Cannot tolerate: 1 AZ outage

Setup:
  ├─ Kubernetes: Pod Anti-Affinity rules
  └─ Terraform: Separate subnets per AZ
```

### 3-Broker Cluster (Minimum)

```
Topology:
  └─ 3 brokers
  └─ Can tolerate: 1 broker failure
  └─ Quorum still requires 2/3

Setup:
  ├─ Kubernetes: 3 replicas
  └─ Terraform: 3 instances
```

### Failover Test

```bash
# Simulate broker failure
kubectl delete pod fastdatabroker-cluster-0

# Kubernetes automatically restarts it
# Check recovery time
kubectl get pods  # Watch for fastdatabroker-cluster-0 restart

# Run client test during failover
python scripts/run_tests.py --category resilience
```

---

## 🆘 Troubleshooting

### Broker won't start

```bash
# Check logs
docker logs fastdatabroker_broker0
# or
kubectl logs fastdatabroker-cluster-0

# Common issues:
# - Port already in use: Kill other service on port 8080
# - Insufficient disk space: Free up space
# - Permission denied: Run with sudo or proper permissions
```

### Slow performance

```bash
# Check metrics
docker stats fastdatabroker_broker0
# or
kubectl describe pod fastdatabroker-cluster-0

# Common issues:
# - CPU maxed out: Scale horizontally
# - Memory pressure: Increase memory limit
# - Network congestion: Check network bandwidth

# Run performance test
python scripts/run_tests.py --category performance
```

### Messages not being consumed

```bash
# Check broker is running
curl http://localhost:8080/health

# Check offsets
python -c "
from postoffice_sdk import Consumer, ClusterClient
client = ClusterClient(['localhost:8080'])
consumer = Consumer(client, 'debug-group')
print(consumer.get_offset())
"

# Check messages exist
python -c "
from postoffice_sdk import Producer, ClusterClient
client = ClusterClient(['localhost:8080'])
producer = Producer(client)
producer.send(b'test', b'test_message')
"
```

### Replica lag too high

```bash
# Check broker connectivity
docker logs fastdatabroker_broker0 | grep replica

# Check network latency
ping broker1  # Should be <5ms

# Restart lagging broker
kubectl delete pod fastdatabroker-cluster-0
# It will resync from leader
```

---

## 📋 Production Checklist

- [ ] Cluster deployed (4 brokers minimum)
- [ ] Monitoring enabled (Prometheus/Grafana)
- [ ] Backups configured (daily)
- [ ] Alerts configured (critical issues)
- [ ] Load testing completed (your throughput expectations)
- [ ] Failover tested (manually shutdown broker)
- [ ] Security reviewed (firewall rules, network policies)
- [ ] Capacity planned (growth expectations)
- [ ] Documentation updated (your deployment notes)
- [ ] Team trained (runbooks, troubleshooting)

---

## 📖 Related Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Get started in 60 seconds
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - How it works
- **[TESTING.md](TESTING.md)** - Test framework
- **[PERFORMANCE.md](PERFORMANCE.md)** - Benchmarks

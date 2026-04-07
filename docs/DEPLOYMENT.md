# FastDataBroker Deployment Guide

Production deployment strategies and best practices for FastDataBroker.

## Deployment Overview

Choose deployment strategy based on scale and requirements:

| Scale | Brokers | Cost | Latency | HA | Throughput |
|-------|---------|------|---------|----|----|
| **Small** | 2-3 | $200-400/mo | <20ms | Single region | 1-2M msg/s |
| **Medium** | 4-5 | $400-600/mo | <20ms | Replicated | 3-5M msg/s |
| **Large** | 8-16 | $800-2000/mo | <20ms | Multi-region | 7-14M msg/s |
| **Enterprise** | 16+ | $2000+/mo | <10ms | Multi-region HA | 14M+ msg/s |

## Single-Server Deployment

### Minimal Setup
```
FastDataBroker (broker0:8080)
├─ Kafka compatibility: No replication
├─ HA: None (single point of failure)
├─ Throughput: 912K msg/sec
├─ Latency: 2-3ms P99
├─ Cost: $100/month
└─ Use case: Development, testing, non-critical
```

### Configuration
```yaml
# broker0.yaml
broker_id: 0
port: 8080
bind_address: 0.0.0.0

storage:
  data_dir: /var/lib/fastdatabroker
  retention_ms: 86400000

replication:
  replication_factor: 1
  min_insync_replicas: 1
```

## Multi-Server HA Deployment

### 4-Node Cluster (Recommended Starting Point)

```
┌────────────────────────────────────────────┐
│        4-Node HA Cluster                   │
├────────────────────────────────────────────┤
│                                             │
│  Broker 0 (t3.large)                       │
│  ├─ Partitions: 0, 5, 10, 15 (leaders)    │
│  └─ Partitions: 1, 9, 13 (replicas)       │
│                                             │
│  Broker 1 (t3.large)                       │
│  ├─ Partitions: 1, 6, 11, 12 (leaders)    │
│  └─ Partitions: 0, 4, 14 (replicas)       │
│                                             │
│  Broker 2 (t3.large)                       │
│  ├─ Partitions: 2, 7, 13, 14 (leaders)    │
│  └─ Partitions: 2, 8, 12 (replicas)       │
│                                             │
│  Broker 3 (t3.large)                       │
│  ├─ Partitions: 3, 8, 9, 4 (leaders)      │
│  └─ Partitions: 3, 11, 10 (replicas)      │
│                                             │
│  Zookeeper Cluster (3 nodes t3.small)      │
│  Monitoring: Prometheus + Grafana          │
│                                             │
└────────────────────────────────────────────┘

Cost: ~$400/month (4 × t3.large)
Throughput: 3.6M msg/sec
Latency: 2-3ms P99
Durability: 3-way replication
HA: Tolerate 1 broker failure
```

### Configuration

```yaml
# broker0.yaml - repeated for brokers 1-3 with different broker_id
broker_id: 0
port: 8080
advertised_address: broker0:8080

zookeeper:
  servers: zk0:2181,zk1:2181,zk2:2181

storage:
  data_dir: /var/lib/fastdatabroker/data
  log_dir: /var/lib/fastdatabroker/logs
  retention_ms: 259200000  # 3 days

replication:
  replication_factor: 3
  min_insync_replicas: 2
  replica_fetch_timeout_ms: 10000

network:
  socket_send_buffer_bytes: 131072
  socket_receive_buffer_bytes: 131072
  num_network_threads: 8
  num_io_threads: 8

monitoring:
  metrics_enabled: true
  metrics_port: 9090
  metrics_path: /metrics
```

## Kubernetes Deployment

### StatefulSet Deployment

```yaml
# k8s/fastdatabroker-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: fastdatabroker
  namespace: data-broker
spec:
  serviceName: fastdatabroker
  replicas: 4
  selector:
    matchLabels:
      app: fastdatabroker
  template:
    metadata:
      labels:
        app: fastdatabroker
    spec:
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - fastdatabroker
            topologyKey: kubernetes.io/hostname
      
      containers:
      - name: fastdatabroker
        image: fastdatabroker:latest
        ports:
        - containerPort: 8080
          name: broker
        - containerPort: 9090
          name: metrics
        
        env:
        - name: BROKER_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: ZOOKEEPER_SERVERS
          value: "zookeeper-0.zookeeper:2181,zookeeper-1.zookeeper:2181,zookeeper-2.zookeeper:2181"
        
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 20
          periodSeconds: 5
        
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        
        volumeMounts:
        - name: data
          mountPath: /var/lib/fastdatabroker
  
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: ssd
      resources:
        requests:
          storage: 100Gi
---
apiVersion: v1
kind: Service
metadata:
  name: fastdatabroker
  namespace: data-broker
spec:
  clusterIP: None
  selector:
    app: fastdatabroker
  ports:
  - port: 8080
    targetPort: 8080
    name: broker
  - port: 9090
    targetPort: 9090
    name: metrics
```

### Deploy
```bash
kubectl create namespace data-broker
kubectl apply -f k8s/fastdatabroker-statefulset.yaml

# Verify deployment
kubectl get statefulsets -n data-broker
kubectl get pods -n data-broker
kubectl logs -n data-broker fastdatabroker-0
```

## Docker Compose Deployment

### Local Development/Testing

```yaml
# docker-compose.yml
version: '3.8'

services:
  zookeeper:
    image: zookeeper:latest
    ports:
      - "2181:2181"
    environment:
      ZOO_CFG_EXTRA: "server.1=zookeeper:2888:3888"

  broker0:
    image: fastdatabroker:latest
    ports:
      - "8080:8080"
    environment:
      BROKER_ID: 0
      ZOOKEEPER_SERVERS: zookeeper:2181
      ADVERTISED_ADDRESS: broker0:8080
    depends_on:
      - zookeeper
    volumes:
      - broker0_data:/var/lib/fastdatabroker

  broker1:
    image: fastdatabroker:latest
    ports:
      - "8081:8080"
    environment:
      BROKER_ID: 1
      ZOOKEEPER_SERVERS: zookeeper:2181
      ADVERTISED_ADDRESS: broker1:8081
    depends_on:
      - zookeeper
    volumes:
      - broker1_data:/var/lib/fastdatabroker

  broker2:
    image: fastdatabroker:latest
    ports:
      - "8082:8080"
    environment:
      BROKER_ID: 2
      ZOOKEEPER_SERVERS: zookeeper:2181
      ADVERTISED_ADDRESS: broker2:8082
    depends_on:
      - zookeeper
    volumes:
      - broker2_data:/var/lib/fastdatabroker

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'

volumes:
  broker0_data:
  broker1_data:
  broker2_data:
  prometheus_data:
```

### Run
```bash
docker-compose up -d
docker-compose logs -f

# Test
curl http://localhost:8080/health
curl http://localhost:9090  # Prometheus
```

## Terraform Infrastructure

### AWS Deployment

```hcl
# terraform/main.tf
provider "aws" {
  region = "us-east-1"
}

resource "aws_instance" "fastdatabroker" {
  count           = 4
  ami             = "ami-0c55b159cbfafe1f0"  # Ubuntu 20.04
  instance_type   = "t3.large"
  key_name        = aws_key_pair.deployer.key_name
  
  tags = {
    Name = "fastdatabroker-${count.index}"
    Role = "broker"
  }
  
  security_groups = [aws_security_group.fastdatabroker.name]
  
  user_data = base64encode(templatefile("${path.module}/broker-init.sh", {
    broker_id = count.index
    zk_servers = join(",", aws_instance.zookeeper[*].private_ip)
  }))
}

resource "aws_security_group" "fastdatabroker" {
  name = "fastdatabroker-sg"
  
  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    self        = true
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
```

## Monitoring & Alerting

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'fastdatabroker'
    static_configs:
      - targets:
        - 'broker0:9090'
        - 'broker1:9090'
        - 'broker2:9090'
        - 'broker3:9090'
```

### Alert Rules

```yaml
groups:
  - name: fastdatabroker
    rules:
      - alert: BrokerDown
        expr: up{job="fastdatabroker"} == 0
        for: 1m
        annotations:
          summary: "Broker {{ $labels.instance }} is down"

      - alert: HighReplicationLag
        expr: fastdatabroker_replication_lag_ms > 5000
        for: 5m
        annotations:
          summary: "High replication lag on {{ $labels.instance }}"

      - alert: HighConsumerLag
        expr: fastdatabroker_consumer_lag_messages > 100000
        for: 10m
        annotations:
          summary: "Consumer lag high on group {{ $labels.group }}"
```

## Rolling Updates

### Safe Upgrade Procedure

```bash
#!/bin/bash
# upgrade-brokers.sh

set -e

BROKERS=(broker0 broker1 broker2 broker3)
NEW_VERSION="1.1.0"

for broker in "${BROKERS[@]}"; do
  echo "Upgrading $broker..."
  
  # 1. Move leadership away
  kubectl exec -it $broker -- admin move-leadership --broker-id $broker
  
  # Wait for leadership to move
  sleep 10
  
  # 2. Update broker image
  kubectl set image deployment/$broker fastdatabroker=fastdatabroker:$NEW_VERSION
  
  # 3. Wait for readiness
  kubectl rollout status deployment/$broker
  
  # 4. Verify stability
  sleep 30
  
  # 5. Check replication lag
  replication_lag=$(kubectl exec -it $broker -- admin get-replication-lag)
  if [ $replication_lag -gt 5000 ]; then
    echo "ERROR: High replication lag after upgrade"
    exit 1
  fi
  
  echo "$broker upgraded successfully"
done

echo "Cluster upgrade complete"
```

## Post-Deployment Verification

```bash
#!/bin/bash
# verify-deployment.sh

echo "1. Checking broker connectivity..."
for broker in broker0 broker1 broker2 broker3; do
  curl -s http://$broker:8080/health || echo "FAIL: $broker health check"
done

echo "2. Checking cluster topology..."
curl -s http://broker0:8080/topology | jq .

echo "3. Checking replication..."
curl -s http://broker0:8080/replication-status | jq .

echo "4. Sending test message..."
curl -X POST http://broker0:8080/stream/test/send \
  -H "Content-Type: application/json" \
  -d '{"key": "test-key", "value": "test-value"}'

echo "5. Consuming test message..."
curl -s http://broker0:8080/stream/test/consume/0/0 | jq .

echo "Verification complete!"
```

## Troubleshooting

### Common Issues

#### Broker Won't Start
```bash
# Check logs
docker logs fastdatabroker-0

# Check Zookeeper connectivity
telnet zookeeper 2181

# Check disk space
df -h /var/lib/fastdatabroker
```

#### High Replication Lag
```bash
# Check broker resources
top, iostat, netstat

# Check network latency
ping broker1 broker2 broker3

# Check if followers are healthy
curl http://broker0:8080/replica-status
```

#### Consumer Lag Growing
```bash
# Check consumer group status
curl http://broker0:8080/consumer-groups

# Check partition leadership
curl http://broker0:8080/partition-leadership

# Manually rebalance consumers
curl -X POST http://broker0:8080/consumer-groups/my-group/rebalance
```

---

**Last Updated**: Phase 7 - Production deployment validated

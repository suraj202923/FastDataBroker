# FastDataBroker Production Deployment Guide 🚀

**Version**: 0.5.0  
**Last Updated**: April 2026  
**Status**: Production-Ready

---

## Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Infrastructure Setup](#infrastructure-setup)
3. [Docker & Registry](#docker--registry)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Load Testing](#load-testing)
6. [Monitoring & Alerting](#monitoring--alerting)
7. [Scaling & Performance](#scaling--performance)
8. [High Availability](#high-availability)
9. [Disaster Recovery](#disaster-recovery)
10. [Troubleshooting](#troubleshooting)
11. [Production Checklist](#production-checklist)

---

## Pre-Deployment Checklist

### Requirements
- [ ] AWS Account with appropriate IAM permissions
- [ ] kubectl 1.28+ installed and configured
- [ ] Terraform 1.0+ installed
- [ ] Docker CE installed
- [ ] Helm 3.0+ installed
- [ ] AWS CLI v2 configured with credentials
- [ ] Docker Hub or AWS ECR account

### Prerequisites

```bash
# Verify Terraform
terraform version

# Verify kubectl
kubectl version --client

# Verify Docker
docker version

# Verify AWS CLI
aws --version
aws sts get-caller-identity  # Verify credentials
```

---

## Infrastructure Setup

### Step 1: Prepare Terraform Configuration

```bash
cd terraform

# Copy example configuration
cp terraform.tfvars.example terraform.tfvars

# Edit configuration with your settings
nano terraform.tfvars
```

### Example terraform.tfvars

```hcl
aws_region              = "us-east-1"
environment             = "production"
FastDataBroker_version      = "0.5.0"
kubernetes_version      = "1.28"
desired_nodes           = 3
max_nodes               = 10
node_instance_types     = ["t3.xlarge", "t3.2xlarge"]
enable_rds              = true
enable_ebs              = true
enable_monitoring       = true
```

### Step 2: Deploy Infrastructure with Terraform

```bash
# Initialize Terraform
terraform init

# Plan deployment
terraform plan -out=tfplan

# Review the plan
less tfplan

# Apply configuration
terraform apply tfplan

# Save outputs
terraform output -json > outputs.json
```

### Step 3: Configure kubectl

```bash
# Get cluster configuration command from Terraform output
export KUBECONFIG=~/.kube/config

# Update kubeconfig
aws eks update-kubeconfig \
  --region us-east-1 \
  --name FastDataBroker-cluster

# Verify cluster access
kubectl cluster-info
kubectl get nodes
```

---

## Docker & Registry

### Step 1: Build Docker Image

```bash
# Build image for production
docker build -t FastDataBroker:0.5.0 \
  --build-arg RUST_BACKTRACE=1 \
  .

# Tag for ECR
export ECR_REPO=$(terraform output -raw ecr_repository_url)
docker tag FastDataBroker:0.5.0 $ECR_REPO:0.5.0
docker tag FastDataBroker:0.5.0 $ECR_REPO:latest
```

### Step 2: Push to ECR

```bash
# Login to ECR
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin $ECR_REPO

# Push image
docker push $ECR_REPO:0.5.0
docker push $ECR_REPO:latest

# Verify image
aws ecr list-images --repository-name FastDataBroker
```

### Step 3: Verify Image Security

```bash
# Enable image scanning in ECR
aws ecr start-image-scan \
  --repository-name FastDataBroker \
  --image-id imageTag=0.5.0

# Check scan results
aws ecr describe-image-scan-findings \
  --repository-name FastDataBroker \
  --image-id imageTag=0.5.0
```

---

## Kubernetes Deployment

### Step 1: Create Namespace & ConfigMaps

```bash
# Apply namespace and configuration
kubectl apply -f kubernetes/01-namespace-config.yaml

# Verify namespace
kubectl get namespace FastDataBroker
kubectl get configmap -n FastDataBroker
```

### Step 2: Create RBAC & Security Policies

```bash
# Apply RBAC and network policies
kubectl apply -f kubernetes/03-rbac-network.yaml

# Verify service account
kubectl get serviceaccount -n FastDataBroker
```

### Step 3: Deploy StatefulSet

```bash
# Update image in statefulset YAML
kubectl set image statefulset/FastDataBroker \
  FastDataBroker=$ECR_REPO:0.5.0 \
  -n FastDataBroker

# Apply deployment
kubectl apply -f kubernetes/02-statefulset-service.yaml

# Monitor rollout
kubectl rollout status statefulset/FastDataBroker -n FastDataBroker

# Check pods
kubectl get pods -n FastDataBroker
kubectl get svc -n FastDataBroker
```

### Step 4: Deploy Monitoring & Autoscaling

```bash
# Apply monitoring stack
kubectl apply -f kubernetes/04-autoscaling-monitoring.yaml

# Verify deployments
kubectl get hpa -n FastDataBroker
kubectl get servicemonitor -n FastDataBroker
```

---

## Load Testing

### Step 1: Setup Load Testing

```bash
# Install Locust
pip install locust

# Or use Bash script for simple testing
chmod +x load_tests/load_test.sh
```

### Step 2: Run Load Tests

```bash
# Baseline test (1000 concurrent users)
locust -f load_tests/locustfile.py \
  --host http://FastDataBroker-lb.FastDataBroker.svc.cluster.local \
  -c 1000 -r 100 -t 300

# Ramp-up test (gradual increase to 5000 users)
locust -f load_tests/locustfile.py \
  --host http://FastDataBroker-lb.FastDataBroker.svc.cluster.local \
  -c 5000 -r 50 -t 600

# Stress test (push to limits)
locust -f load_tests/locustfile.py \
  --host http://FastDataBroker-lb.FastDataBroker.svc.cluster.local \
  -c 10000 -r 200 -t 900
```

### Step 3: Monitor During Load Test

```bash
# In separate terminal, monitor metrics
while true; do
  kubectl top nodes -n FastDataBroker
  kubectl top pods -n FastDataBroker
  sleep 10
done

# Monitor logs
kubectl logs -f deployment/FastDataBroker -n FastDataBroker --all-containers
```

---

## Monitoring & Alerting

### Step 1: Deploy Prometheus

```bash
# Add Prometheus Helm repository
helm repo add prometheus-community \
  https://prometheus-community.github.io/helm-charts
helm repo update

# Install Prometheus
helm install prometheus prometheus-community/kube-prometheus-stack \
  -f monitoring/prometheus-values.yaml \
  --namespace observability \
  --create-namespace
```

### Step 2: Configure Alerts

```bash
# Apply alert rules
kubectl apply -f kubernetes/04-autoscaling-monitoring.yaml

# Verify alerts
kubectl get prometheusrule -n FastDataBroker
```

### Step 3: Access Prometheus Dashboard

```bash
# Port-forward Prometheus
kubectl port-forward -n observability \
  svc/prometheus-kube-prometheus-prometheus 9090:9090

# Access at http://localhost:9090
```

### Step 4: Configure Grafana

```bash
# Get Grafana admin password
kubectl get secret -n observability \
  prometheus-grafana -o jsonpath="{.data.admin-password}" | base64 --decode

# Port-forward Grafana
kubectl port-forward -n observability \
  svc/prometheus-grafana 3000:80

# Access at http://localhost:3000
```

---

## Scaling & Performance

### Manual Scaling

```bash
# Scale StatefulSet
kubectl scale statefulset/FastDataBroker \
  --replicas=5 \
  -n FastDataBroker

# Monitor scaling
kubectl rollout status statefulset/FastDataBroker -n FastDataBroker
```

### Auto-Scaling Configuration

```bash
# Check HPA status
kubectl get hpa -n FastDataBroker

# Modify HPA settings
kubectl autoscale statefulset/FastDataBroker \
  --min=3 --max=10 \
  --cpu-percent=70 \
  -n FastDataBroker
```

### Performance Tuning

```bash
# Increase message throughput
kubectl set env statefulset/FastDataBroker \
  RUST_LOG=warn \
  -n FastDataBroker

# Disable JSON logging in production
kubectl set env statefulset/FastDataBroker \
  LOGGING_FORMAT=text \
  -n FastDataBroker

# Scroll rollout
kubectl rollout restart statefulset/FastDataBroker -n FastDataBroker
```

---

## High Availability

### Multi-Region Setup

```bash
# Deploy to multiple regions
terraform apply -var="aws_region=us-west-2" \
  -state="terraform-west.tfstate"

terraform apply -var="aws_region=eu-west-1" \
  -state="terraform-eu.tfstate"
```

### Multi-AZ Verification

```bash
# Verify pods are spread across AZs
kubectl get pods -n FastDataBroker \
  -o wide \
  -L topology.kubernetes.io/zone

# Expected: pods distributed across 3 AZs
```

### Failover Testing

```bash
# Simulate node failure
kubectl drain <node-name> --delete-emptydir-data

# Pods should reschedule to healthy nodes
kubectl get pods -n FastDataBroker -w

# Uncordon node
kubectl uncordon <node-name>
```

---

## Disaster Recovery

### Backup Strategy

```bash
# Enable automated EBS snapshots
aws ec2 create-scheduled-actions \
  --scheduled-action-name FastDataBroker-backup \
  --recurrence "0 2 * * *" \
  --action-type CreateSnapshot

# Backup RDS database
aws rds create-db-snapshot \
  --db-instance-identifier FastDataBroker-db \
  --db-snapshot-identifier FastDataBroker-backup-$(date +%Y%m%d)
```

### Restore Procedures

```bash
# Restore from EBS snapshot
aws ec2 create-volume \
  --snapshot-id snap-xxxxx \
  --availability-zone us-east-1a

# Restore RDS from snapshot
aws rds restore-db-instance-from-db-snapshot \
  --db-instance-identifier FastDataBroker-restored \
  --db-snapshot-identifier FastDataBroker-backup-20260407
```

### Data Persistence Verification

```bash
# Check persistent volume status
kubectl get pv -n FastDataBroker
kubectl get pvc -n FastDataBroker

# Verify data integrity
kubectl exec -it FastDataBroker-0 -n FastDataBroker -- \
  sh -c "ls -lah /data"
```

---

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod FastDataBroker-0 -n FastDataBroker

# Check logs
kubectl logs FastDataBroker-0 -n FastDataBroker
kubectl logs FastDataBroker-0 -n FastDataBroker --previous

# Check events
kubectl get events -n FastDataBroker
```

### High Latency

```bash
# Check resource usage
kubectl top pod FastDataBroker-0 -n FastDataBroker

# Check network performance
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  iperf3 -c <other-pod-ip>

# Check disk I/O
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  iostat -x 1 5
```

### Message Loss

```bash
# Check message queue depth
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  curl localhost:6380/metrics | grep queue

# Verify PVC is mounted
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  mount | grep data

# Check disk space
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  df -h /data
```

### Circuit Breaker Trips

```bash
# Check circuit breaker status
kubectl exec FastDataBroker-0 -n FastDataBroker -- \
  curl localhost:6380/metrics | grep circuit_breaker

# Check service health
kubectl get endpoints FastDataBroker -n FastDataBroker

# Check dependent services
kubectl get svc -A | grep -E "jaeger|prometheus|rds"
```

---

## Production Checklist

Before going live, verify:

- [ ] All 3+ replicas are running and healthy
- [ ] Load test completes without data loss
- [ ] Metrics are being collected in Prometheus
- [ ] Alerts are configured and tested
- [ ] Backups are scheduled and tested
- [ ] HPA is working correctly
- [ ] Monitoring dashboards are configured
- [ ] Multi-region failover is tested
- [ ] Security scans pass with no critical issues
- [ ] Network policies are properly configured
- [ ] RBAC permissions are minimal and correct
- [ ] SSL/TLS certificates are installed
- [ ] DNS is pointing to load balancer
- [ ] Rate limiting is configured
- [ ] Circuit breaker thresholds are tuned
- [ ] Encryption keys are securely managed
- [ ] Database backups are operational
- [ ] Recovery Time Objective (RTO) tested: <5 min
- [ ] Recovery Point Objective (RPO) tested: <1 min
- [ ] Runbooks are documented and rehearsed

---

## Performance Targets

### Expected Metrics (3-node cluster)

| Metric | Target | Unit |
|--------|--------|------|
| Throughput | 1M+ | msg/sec |
| P50 Latency | <50 | ms |
| P95 Latency | <200 | ms |
| P99 Latency | <1000 | ms |
| Error Rate | <0.1 | % |
| CPU Usage | 60-70 | % |
| Memory Usage | 70-80 | % |
| Disk Usage | <80 | % |
| Network Latency | <10 | ms (inter-node) |

---

## Support & References

- **Documentation**: https://github.com/suraj202923/rst_queue_FastDataBroker
- **Issues**: https://github.com/suraj202923/rst_queue_FastDataBroker/issues
- **Kubernetes Docs**: https://kubernetes.io/docs/
- **Terraform Docs**: https://www.terraform.io/docs/
- **AWS EKS Guide**: https://docs.aws.amazon.com/eks/

---

## Glossary

- **RTO**: Recovery Time Objective - Maximum acceptable downtime
- **RPO**: Recovery Point Objective - Maximum acceptable data loss
- **HPA**: Horizontal Pod Autoscaler
- **PVC**: PersistentVolumeClaim
- **RBAC**: Role-Based Access Control
- **ECR**: Elastic Container Registry
- **EKS**: Elastic Kubernetes Service

---

**Last Updated**: April 7, 2026  
**Maintained By**: FastDataBroker Team  
**Version**: 1.0

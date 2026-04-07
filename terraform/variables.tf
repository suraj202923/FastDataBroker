variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "FastDataBroker_version" {
  description = "FastDataBroker version"
  type        = string
  default     = "0.5.0"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "kubernetes_version" {
  description = "Kubernetes version for EKS"
  type        = string
  default     = "1.28"
}

variable "node_instance_types" {
  description = "EC2 instance types for worker nodes"
  type        = list(string)
  default     = ["t3.xlarge", "t3.2xlarge"]
}

variable "node_disk_size" {
  description = "EBS volume size for nodes (GB)"
  type        = number
  default     = 100
}

variable "desired_nodes" {
  description = "Desired number of worker nodes"
  type        = number
  default     = 3
  validation {
    condition     = var.desired_nodes >= 3 && var.desired_nodes <= 20
    error_message = "Desired nodes must be between 3 and 20 for HA."
  }
}

variable "min_nodes" {
  description = "Minimum number of worker nodes"
  type        = number
  default     = 3
}

variable "max_nodes" {
  description = "Maximum number of worker nodes"
  type        = number
  default     = 10
}

variable "public_access_cidrs" {
  description = "CIDR blocks for public API access"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "enable_rds" {
  description = "Enable RDS for persistent storage"
  type        = bool
  default     = false
}

variable "rds_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.r6i.xlarge"
}

variable "rds_allocated_storage" {
  description = "RDS allocated storage (GB)"
  type        = number
  default     = 100
}

variable "enable_ebs" {
  description = "Enable EBS volumes for data persistence"
  type        = bool
  default     = true
}

variable "ebs_volume_size" {
  description = "EBS volume size (GB)"
  type        = number
  default     = 500
}

variable "enable_monitoring" {
  description = "Enable CloudWatch monitoring"
  type        = bool
  default     = true
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 30
}

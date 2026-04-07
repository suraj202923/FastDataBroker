# VPC for FastDataBroker
resource "aws_vpc" "FastDataBroker" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "FastDataBroker-vpc"
  }
}

# Internet Gateway
resource "aws_internet_gateway" "FastDataBroker" {
  vpc_id = aws_vpc.FastDataBroker.id

  tags = {
    Name = "FastDataBroker-igw"
  }

  depends_on = [aws_vpc.FastDataBroker]
}

# Public Subnets (3 AZs for HA)
resource "aws_subnet" "public" {
  count                   = 3
  vpc_id                  = aws_vpc.FastDataBroker.id
  cidr_block              = "10.0.${count.index + 1}.0/24"
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "FastDataBroker-public-${count.index + 1}"
    Type = "Public"
  }
}

# Private Subnets (3 AZs for HA)
resource "aws_subnet" "private" {
  count             = 3
  vpc_id            = aws_vpc.FastDataBroker.id
  cidr_block        = "10.0.${count.index + 11}.0/24"
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name = "FastDataBroker-private-${count.index + 1}"
    Type = "Private"
  }
}

# Elastic IPs for NAT Gateways
resource "aws_eip" "nat" {
  count  = 3
  domain = "vpc"

  tags = {
    Name = "FastDataBroker-nat-${count.index + 1}"
  }

  depends_on = [aws_internet_gateway.FastDataBroker]
}

# NAT Gateways for private subnet egress
resource "aws_nat_gateway" "FastDataBroker" {
  count         = 3
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  tags = {
    Name = "FastDataBroker-nat-${count.index + 1}"
  }

  depends_on = [aws_internet_gateway.FastDataBroker]
}

# Route table for public subnets
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.FastDataBroker.id

  route {
    cidr_block      = "0.0.0.0/0"
    gateway_id      = aws_internet_gateway.FastDataBroker.id
  }

  tags = {
    Name = "FastDataBroker-public-rt"
  }
}

# Route table associations for public subnets
resource "aws_route_table_association" "public" {
  count          = 3
  route_table_id = aws_route_table.public.id
  subnet_id      = aws_subnet.public[count.index].id
}

# Route tables for private subnets (one per AZ for NAT)
resource "aws_route_table" "private" {
  count  = 3
  vpc_id = aws_vpc.FastDataBroker.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.FastDataBroker[count.index].id
  }

  tags = {
    Name = "FastDataBroker-private-rt-${count.index + 1}"
  }
}

# Route table associations for private subnets
resource "aws_route_table_association" "private" {
  count          = 3
  route_table_id = aws_route_table.private[count.index].id
  subnet_id      = aws_subnet.private[count.index].id
}

# Security Group for EKS control plane
resource "aws_security_group" "FastDataBroker_control_plane" {
  name        = "FastDataBroker-control-plane"
  description = "Security group for FastDataBroker EKS control plane"
  vpc_id      = aws_vpc.FastDataBroker.id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "FastDataBroker-control-plane-sg"
  }
}

# Security Group for EKS worker nodes
resource "aws_security_group" "FastDataBroker_node" {
  name        = "FastDataBroker-node"
  description = "Security group for FastDataBroker EKS nodes"
  vpc_id      = aws_vpc.FastDataBroker.id

  ingress {
    from_port   = 0
    to_port     = 65535
    protocol    = "tcp"
    self        = true
  }

  ingress {
    from_port   = 0
    to_port     = 65535
    protocol    = "udp"
    self        = true
  }

  ingress {
    from_port       = 6379
    to_port         = 6381
    protocol        = "tcp"
    security_groups = [aws_security_group.FastDataBroker_control_plane.id]
  }

  ingress {
    from_port       = 6379
    to_port         = 6379
    protocol        = "udp"
    security_groups = [aws_security_group.FastDataBroker_control_plane.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "FastDataBroker-node-sg"
  }
}

# Data source for availability zones
data "aws_availability_zones" "available" {
  state = "available"
}

# EKS Cluster
resource "aws_eks_cluster" "FastDataBroker" {
  name            = "FastDataBroker-cluster"
  version         = var.kubernetes_version
  role_arn        = aws_iam_role.eks_cluster_role.arn
  vpc_config {
    subnet_ids              = concat(aws_subnet.public[*].id, aws_subnet.private[*].id)
    endpoint_private_access = true
    endpoint_public_access  = true
    public_access_cidrs     = var.public_access_cidrs
    security_group_ids      = [aws_security_group.FastDataBroker_control_plane.id]
  }

  depends_on = [aws_iam_role_policy_attachment.eks_cluster_policy]

  tags = {
    Name = "FastDataBroker-cluster"
  }
}

# IAM Role for EKS Cluster
resource "aws_iam_role" "eks_cluster_role" {
  name = "FastDataBroker-eks-cluster-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "eks.amazonaws.com"
        }
      }
    ]
  })
}

# Attach cluster policy
resource "aws_iam_role_policy_attachment" "eks_cluster_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSClusterPolicy"
  role       = aws_iam_role.eks_cluster_role.name
}

# EKS Node Group
resource "aws_eks_node_group" "FastDataBroker" {
  cluster_name    = aws_eks_cluster.FastDataBroker.name
  node_group_name = "FastDataBroker-nodes"
  node_role_arn   = aws_iam_role.eks_node_role.arn
  subnet_ids      = aws_subnet.private[*].id

  version         = var.kubernetes_version
  instance_types  = var.node_instance_types
  disk_size       = var.node_disk_size
  desired_size    = var.desired_nodes
  min_size        = var.min_nodes
  max_size        = var.max_nodes

  capacity_type = "ON_DEMAND"

  # Tags for node autoscaler
  labels = {
    Environment = var.environment
    NodeGroup   = "FastDataBroker"
  }

  tags = {
    Name = "FastDataBroker-nodes"
  }

  depends_on = [
    aws_iam_role_policy_attachment.eks_worker_node_policy,
    aws_iam_role_policy_attachment.eks_cni_policy,
    aws_iam_role_policy_attachment.eks_container_registry_policy,
  ]
}

# IAM Role for EKS Nodes
resource "aws_iam_role" "eks_node_role" {
  name = "FastDataBroker-eks-node-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })
}

# Attach policies to node role
resource "aws_iam_role_policy_attachment" "eks_worker_node_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKSWorkerNodePolicy"
  role       = aws_iam_role.eks_node_role.name
}

resource "aws_iam_role_policy_attachment" "eks_cni_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEKS_CNI_Policy"
  role       = aws_iam_role.eks_node_role.name
}

resource "aws_iam_role_policy_attachment" "eks_container_registry_policy" {
  policy_arn = "arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly"
  role       = aws_iam_role.eks_node_role.name
}

# ECR Repository for FastDataBroker image
resource "aws_ecr_repository" "FastDataBroker" {
  name                 = "FastDataBroker"
  image_tag_mutability = "IMMUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  encryption_configuration {
    encryption_type = "KMS"
  }

  tags = {
    Name = "FastDataBroker"
  }
}

# ECR Lifecycle Policy
resource "aws_ecr_lifecycle_policy" "FastDataBroker" {
  repository = aws_ecr_repository.FastDataBroker.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Keep last 10 images"
        selection = {
          tagStatus       = "tagged"
          tagPrefixList   = ["v"]
          countType       = "imageCountMoreThan"
          countNumber     = 10
        }
        action = {
          type = "expire"
        }
      },
      {
        rulePriority = 2
        description  = "Remove untagged images"
        selection = {
          tagStatus   = "untagged"
          countType   = "sinceImagePushed"
          countUnit   = "days"
          countNumber = 30
        }
        action = {
          type = "expire"
        }
      }
    ]
  })
}

# RDS for persistent storage (optional)
resource "aws_db_instance" "FastDataBroker" {
  count = var.enable_rds ? 1 : 0

  identifier     = "FastDataBroker-db"
  engine         = "postgres"
  engine_version = "15.3"
  instance_class = var.rds_instance_class

  allocated_storage     = var.rds_allocated_storage
  storage_type          = "gp3"
  storage_encrypted     = true
  iops                  = 3000
  performance_insights_enabled = true

  db_name  = "FastDataBroker"
  username = "admin"
  password = random_password.db_password[0].result

  multi_az               = true
  publicly_accessible    = false
  skip_final_snapshot    = false
  final_snapshot_identifier = "FastDataBroker-final-snapshot-${formatdate("YYYY-MM-DD-hhmm", timestamp())}"

  vpc_security_group_ids = [aws_security_group.FastDataBroker_rds[0].id]
  db_subnet_group_name   = aws_db_subnet_group.FastDataBroker[0].name

  backup_retention_period = 30
  backup_window          = "03:00-04:00"
  maintenance_window     = "mon:04:00-mon:05:00"

  enabled_cloudwatch_logs_exports = ["postgresql"]

  tags = {
    Name = "FastDataBroker-db"
  }
}

# Generate secure RDS password
resource "random_password" "db_password" {
  count   = var.enable_rds ? 1 : 0
  length  = 32
  special = true
}

# Security Group for RDS
resource "aws_security_group" "FastDataBroker_rds" {
  count       = var.enable_rds ? 1 : 0
  name        = "FastDataBroker-rds"
  description = "Security group for FastDataBroker RDS"
  vpc_id      = aws_vpc.FastDataBroker.id

  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.FastDataBroker_node.id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "FastDataBroker-rds-sg"
  }
}

# DB Subnet Group
resource "aws_db_subnet_group" "FastDataBroker" {
  count           = var.enable_rds ? 1 : 0
  name            = "FastDataBroker-db-subnet"
  subnet_ids      = aws_subnet.private[*].id
  tags = {
    Name = "FastDataBroker-db-subnet"
  }
}

# EBS Volume for persistent storage
resource "aws_ebs_volume" "FastDataBroker_data" {
  count             = var.enable_ebs ? 1 : 0
  availability_zone = data.aws_availability_zones.available.names[0]
  size              = var.ebs_volume_size
  type              = "gp3"
  encrypted         = true
  iops              = 3000
  throughput        = 125

  tags = {
    Name = "FastDataBroker-data"
  }
}

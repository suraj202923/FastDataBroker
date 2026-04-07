terraform {
  required_version = ">= 1.0"
  
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.25"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.12"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  # Backend configuration for state management
  backend "s3" {
    bucket         = "FastDataBroker-terraform-state"
    key            = "FastDataBroker/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-locks"
  }
}

provider "kubernetes" {
  host                   = aws_eks_cluster.FastDataBroker.endpoint
  cluster_ca_certificate = base64decode(aws_eks_cluster.FastDataBroker.certificate_authority[0].data)
  token                  = data.aws_eks_cluster_auth.FastDataBroker.token

  load_config_file = false
}

provider "helm" {
  kubernetes {
    host                   = aws_eks_cluster.FastDataBroker.endpoint
    cluster_ca_certificate = base64decode(aws_eks_cluster.FastDataBroker.certificate_authority[0].data)
    token                  = data.aws_eks_cluster_auth.FastDataBroker.token
    load_config_file       = false
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "FastDataBroker"
      Environment = var.environment
      ManagedBy   = "Terraform"
      Version     = var.FastDataBroker_version
    }
  }
}

data "aws_eks_cluster_auth" "FastDataBroker" {
  name = aws_eks_cluster.FastDataBroker.name
}

output "cluster_name" {
  description = "EKS cluster name"
  value       = aws_eks_cluster.FastDataBroker.name
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = aws_eks_cluster.FastDataBroker.endpoint
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the cluster"
  value       = aws_security_group.FastDataBroker_control_plane.id
}

output "cluster_iam_role_arn" {
  description = "IAM role ARN of the EKS cluster"
  value       = aws_eks_cluster.FastDataBroker.role_arn
}

output "node_group_id" {
  description = "EKS node group id"
  value       = aws_eks_node_group.FastDataBroker.id
}

output "node_group_arn" {
  description = "Amazon Resource Name (ARN) of the EKS Node Group"
  value       = aws_eks_node_group.FastDataBroker.arn
}

output "node_security_group_id" {
  description = "Security group ID attached to the EKS nodes"
  value       = aws_security_group.FastDataBroker_node.id
}

output "ecr_repository_url" {
  description = "URL of the ECR repository"
  value       = aws_ecr_repository.FastDataBroker.repository_url
}

output "ecr_registry_id" {
  description = "The registry ID where the repository was created"
  value       = aws_ecr_repository.FastDataBroker.registry_id
}

output "vpc_id" {
  description = "The ID of the VPC"
  value       = aws_vpc.FastDataBroker.id
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs"
  value       = aws_subnet.private[*].id
}

output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = try(aws_db_instance.FastDataBroker[0].endpoint, null)
}

output "rds_database_name" {
  description = "RDS database name"
  value       = try(aws_db_instance.FastDataBroker[0].db_name, null)
}

output "rds_username" {
  description = "RDS master username"
  value       = try(aws_db_instance.FastDataBroker[0].username, null)
  sensitive   = true
}

output "rds_password" {
  description = "RDS master password"
  value       = try(random_password.db_password[0].result, null)
  sensitive   = true
}

output "ebs_volume_id" {
  description = "EBS volume ID for data persistence"
  value       = try(aws_ebs_volume.FastDataBroker_data[0].id, null)
}

output "kubectl_config" {
  description = "Configure kubectl with easiest approach"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${aws_eks_cluster.FastDataBroker.name}"
}

output "deployment_summary" {
  description = "Deployment summary"
  value = {
    cluster_name              = aws_eks_cluster.FastDataBroker.name
    cluster_endpoint          = aws_eks_cluster.FastDataBroker.endpoint
    region                    = var.aws_region
    node_group_size           = var.desired_nodes
    ecr_repository_url        = aws_ecr_repository.FastDataBroker.repository_url
    kubernetes_version        = var.kubernetes_version
    environment               = var.environment
    FastDataBroker_version        = var.FastDataBroker_version
  }
}

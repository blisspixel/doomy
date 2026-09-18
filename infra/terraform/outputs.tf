# fragr Terraform Outputs

output "project_id" {
  description = "GCP project ID"
  value       = var.project_id
}

output "region" {
  description = "GCP region"
  value       = var.region
}

output "zone" {
  description = "GCP zone"
  value       = var.zone
}

output "instance_name" {
  description = "VM instance name"
  value       = google_compute_instance.game_server.name
}

output "instance_id" {
  description = "VM instance ID"
  value       = google_compute_instance.game_server.instance_id
}

output "external_ip" {
  description = "Ephemeral external IP address (changes on stop/start)"
  value       = google_compute_instance.game_server.network_interface[0].access_config[0].nat_ip
}

output "internal_ip" {
  description = "Internal IP address"
  value       = google_compute_instance.game_server.network_interface[0].network_ip
}

output "service_account_email" {
  description = "Service account email for the VM"
  value       = google_service_account.game_server.email
}

output "network_name" {
  description = "VPC network name"
  value       = google_compute_network.vpc.name
}

output "subnet_name" {
  description = "Subnet name"
  value       = google_compute_subnetwork.subnet.name
}

output "ssh_command" {
  description = "SSH command to connect via IAP"
  value       = "gcloud compute ssh ${google_compute_instance.game_server.name} --zone=${var.zone} --tunnel-through-iap"
}

output "game_server_address" {
  description = "Game server address (IP:PORT)"
  value       = "${google_compute_instance.game_server.network_interface[0].access_config[0].nat_ip}:${var.game_port}"
}

output "cloud_run_url" {
  description = "Cloud Run adapter URL (if enabled)"
  value       = var.enable_cloud_run_adapter ? google_cloud_run_v2_service.adapter[0].uri : "Not enabled"
}

output "estimated_monthly_cost" {
  description = "Estimated monthly cost in USD"
  value       = "$0.00 (Always Free tier)"
}

output "cost_warnings" {
  description = "Potential cost warnings"
  value = [
    "Network egress beyond 1 GB/month: ~$0.12/GB",
    "Stopping/starting VM changes external IP",
    "Monitor billing at: https://console.cloud.google.com/billing"
  ]
}

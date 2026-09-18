# fragr Terraform Variables

variable "project_id" {
  description = "GCP project ID"
  type        = string

  validation {
    condition     = length(var.project_id) > 0
    error_message = "Project ID must not be empty."
  }
}

variable "region" {
  description = "GCP region for resources (must be Always Free eligible: us-west1, us-central1, us-east1)"
  type        = string
  default     = "us-west1"

  validation {
    condition     = contains(["us-west1", "us-central1", "us-east1"], var.region)
    error_message = "Region must be us-west1, us-central1, or us-east1 for Always Free tier."
  }
}

variable "zone" {
  description = "GCP zone within the region"
  type        = string
  default     = "us-west1-a"
}

variable "instance_name" {
  description = "Name for the VM instance"
  type        = string
  default     = "fragr-game-server"

  validation {
    condition     = can(regex("^[a-z]([-a-z0-9]*[a-z0-9])?$", var.instance_name))
    error_message = "Instance name must start with lowercase letter, contain only lowercase letters, numbers, and hyphens."
  }
}

variable "machine_type" {
  description = "Machine type for VM (locked to e2-micro for Always Free)"
  type        = string
  default     = "e2-micro"

  validation {
    condition     = var.machine_type == "e2-micro"
    error_message = "Machine type must be e2-micro for Always Free tier. Change requires explicit override."
  }
}

variable "boot_disk_size_gb" {
  description = "Boot disk size in GB (must be <= 30 for Always Free)"
  type        = number
  default     = 20

  validation {
    condition     = var.boot_disk_size_gb > 0 && var.boot_disk_size_gb <= 30
    error_message = "Boot disk size must be between 1 and 30 GB for Always Free tier."
  }
}

variable "game_port" {
  description = "Game server port (TCP for WebSocket, UDP for future renet)"
  type        = number
  default     = 6767

  validation {
    condition     = var.game_port > 1024 && var.game_port < 65536
    error_message = "Game port must be between 1024 and 65535."
  }
}

variable "network_cidr" {
  description = "CIDR range for VPC network"
  type        = string
  default     = "10.0.0.0/24"

  validation {
    condition     = can(cidrhost(var.network_cidr, 0))
    error_message = "Network CIDR must be a valid IPv4 CIDR block."
  }
}

variable "ssh_source_ranges" {
  description = "Additional source IP ranges for SSH access (IAP is always enabled)"
  type        = list(string)
  default     = []
}

variable "enable_cloud_run_adapter" {
  description = "Enable Cloud Run HTTP adapter (min_instances=0)"
  type        = bool
  default     = false
}

variable "cloud_run_region" {
  description = "Region for Cloud Run service"
  type        = string
  default     = "us-west1"
}

variable "labels" {
  description = "Labels to apply to resources"
  type        = map(string)
  default = {
    project    = "fragr"
    managed_by = "terraform"
  }
}

# fragr Infrastructure - Main Configuration
# Zero-cost GCP deployment using Always Free tier

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }

  # Uncomment after first apply to enable remote state
  # backend "gcs" {
  #   bucket = "YOUR-PROJECT-ID-tfstate"
  #   prefix = "fragr/terraform/state"
  # }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

# Enable required APIs
resource "google_project_service" "compute" {
  project = var.project_id
  service = "compute.googleapis.com"

  disable_on_destroy = false
}

resource "google_project_service" "iap" {
  project = var.project_id
  service = "iap.googleapis.com"

  disable_on_destroy = false
}

resource "google_project_service" "cloud_run" {
  count   = var.enable_cloud_run_adapter ? 1 : 0
  project = var.project_id
  service = "run.googleapis.com"

  disable_on_destroy = false
}

# VPC Network
resource "google_compute_network" "vpc" {
  name                    = "fragr-vpc"
  auto_create_subnetworks = false
  routing_mode            = "REGIONAL"

  depends_on = [google_project_service.compute]
}

# Subnet
resource "google_compute_subnetwork" "subnet" {
  name          = "fragr-subnet"
  ip_cidr_range = var.network_cidr
  region        = var.region
  network       = google_compute_network.vpc.id

  private_ip_google_access = true
}

# Firewall: Allow game port from all (TCP for WebSocket, UDP for future renet)
resource "google_compute_firewall" "game_port_tcp" {
  name    = "fragr-allow-game-tcp"
  network = google_compute_network.vpc.name

  allow {
    protocol = "tcp"
    ports    = [tostring(var.game_port)]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["fragr-game-server"]

  description = "Allow TCP game traffic (WebSocket) on port ${var.game_port}"
}

resource "google_compute_firewall" "game_port_udp" {
  name    = "fragr-allow-game-udp"
  network = google_compute_network.vpc.name

  allow {
    protocol = "udp"
    ports    = [tostring(var.game_port)]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["fragr-game-server"]

  description = "Allow UDP game traffic (future renet) on port ${var.game_port}"
}

# Firewall: Allow IAP SSH
resource "google_compute_firewall" "iap_ssh" {
  name    = "fragr-allow-iap-ssh"
  network = google_compute_network.vpc.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  # IAP IP range for TCP forwarding
  source_ranges = ["35.235.240.0/20"]
  target_tags   = ["fragr-game-server"]

  description = "Allow SSH via Identity-Aware Proxy"
}

# Optional: Allow SSH from specific IPs
resource "google_compute_firewall" "ssh_custom" {
  count   = length(var.ssh_source_ranges) > 0 ? 1 : 0
  name    = "fragr-allow-ssh-custom"
  network = google_compute_network.vpc.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = var.ssh_source_ranges
  target_tags   = ["fragr-game-server"]

  description = "Allow SSH from specific IP ranges"
}

# Service Account for VM
resource "google_service_account" "game_server" {
  account_id   = "fragr-game-server"
  display_name = "fragr Game Server Service Account"
  description  = "Service account for fragr game server VM with minimal permissions"
}

# Grant minimal permissions to service account
resource "google_project_iam_member" "game_server_log_writer" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.game_server.email}"
}

resource "google_project_iam_member" "game_server_metric_writer" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.game_server.email}"
}

# VM Instance
resource "google_compute_instance" "game_server" {
  name         = var.instance_name
  machine_type = var.machine_type
  zone         = var.zone

  tags = ["fragr-game-server"]

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
      size  = var.boot_disk_size_gb
      type  = "pd-standard"
    }
  }

  network_interface {
    network    = google_compute_network.vpc.id
    subnetwork = google_compute_subnetwork.subnet.id

    # Ephemeral external IP (no static IP cost)
    access_config {
      # Ephemeral IP
    }
  }

  service_account {
    email  = google_service_account.game_server.email
    scopes = ["cloud-platform"]
  }

  metadata = {
    enable-oslogin = "TRUE"
  }

  metadata_startup_script = <<-EOF
    #!/bin/bash
    set -e
    
    # Update system
    apt-get update
    apt-get install -y curl
    
    # Install Docker (for containerized server if needed)
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    
    # Create directory for game server
    mkdir -p /opt/fragr
    chown -R root:root /opt/fragr
    
    # Placeholder: Server binary deployment
    # In production, copy binary from GCS or artifact registry
    echo "Game server ready for deployment" > /opt/fragr/status.txt
  EOF

  labels = var.labels

  lifecycle {
    ignore_changes = [
      metadata_startup_script,
    ]
  }

  depends_on = [
    google_project_service.compute,
    google_compute_subnetwork.subnet,
  ]
}

# Optional: Cloud Run HTTP Adapter
resource "google_service_account" "cloud_run_adapter" {
  count        = var.enable_cloud_run_adapter ? 1 : 0
  account_id   = "fragr-cloud-run-adapter"
  display_name = "fragr Cloud Run Adapter Service Account"
  description  = "Service account for Cloud Run HTTP adapter"
}

resource "google_cloud_run_v2_service" "adapter" {
  count    = var.enable_cloud_run_adapter ? 1 : 0
  name     = "fragr-adapter"
  location = var.cloud_run_region

  template {
    service_account = google_service_account.cloud_run_adapter[0].email

    scaling {
      min_instance_count = 0
      max_instance_count = 1
    }

    containers {
      # Placeholder image - replace with actual adapter image
      image = "us-docker.pkg.dev/cloudrun/container/hello"

      ports {
        container_port = 8080
      }

      resources {
        limits = {
          cpu    = "1"
          memory = "512Mi"
        }
      }

      env {
        name  = "GAME_SERVER_HOST"
        value = google_compute_instance.game_server.network_interface[0].network_ip
      }

      env {
        name  = "GAME_SERVER_PORT"
        value = tostring(var.game_port)
      }
    }
  }

  labels = var.labels

  depends_on = [
    google_project_service.cloud_run,
  ]
}

# Cloud Run: No public access (requires authentication)
# Do NOT add allUsers invoker permission

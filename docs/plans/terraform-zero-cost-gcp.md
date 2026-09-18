# Plan: Terraform Infrastructure for Zero-Cost GCP Deployment

**Status:** Implemented (PR #5, not applied)  
**Date:** 2026-09-18  
**Spend:** $0 (plan only, no apply)

## Goal

Create Terraform infrastructure-as-code for deploying fragr game servers on GCP Always Free tier, targeting $0/month baseline cost.

## Non-Goals

- Terraform apply in this phase (requires Nick/Chief approval)
- Static IP allocation (costs ~$2.88/month)
- Load balancers (costs ~$18/month minimum)
- Managed instance groups or GKE (exceeds Always Free tier)
- CI/CD pipeline setup (Gitty owns)
- Production deployment configuration

## Architecture Impact

### New Components

- `infra/` directory with Terraform modules
- `infra/docs/` for infrastructure-specific documentation
- `infra/terraform/` with validated Terraform configuration

### No Changes To

- Server code (no modifications required)
- Client code (connects to IP:PORT, unchanged)
- Agent adapter (no infrastructure dependencies)
- Protocol definitions (unchanged)

## Terraform Resources

### Compute

- **VM Instance:** `google_compute_instance.game_server`
  - Machine type: `e2-micro` (2 vCPU, 1 GB memory)
  - Region: `us-west1` (variable, restricted to Always Free zones)
  - Boot disk: 20 GB standard persistent (under 30 GB free tier)
  - Image: Debian 12 (stable, well-supported)
  - Ephemeral external IP (no static IP resource)
  - Startup script: Install Docker, create /opt/fragr directory

### Networking

- **VPC:** `google_compute_network.vpc`
  - Name: `fragr-vpc`
  - Regional routing
  - Manual subnets (no auto-create)

- **Subnet:** `google_compute_subnetwork.subnet`
  - CIDR: 10.0.0.0/24 (variable)
  - Private Google access enabled

- **Firewall Rules:**
  - `google_compute_firewall.game_port` - Allow UDP 7777 from 0.0.0.0/0
  - `google_compute_firewall.iap_ssh` - Allow TCP 22 from 35.235.240.0/20 (IAP)
  - `google_compute_firewall.ssh_custom` - Optional: Allow TCP 22 from specific IPs

### IAM

- **Service Account:** `google_service_account.game_server`
  - ID: `fragr-game-server`
  - Roles: `roles/logging.logWriter`, `roles/monitoring.metricWriter`
  - Minimal permissions (no compute admin, no storage access)

- **OS Login:** Enabled for IAM-based SSH access

### Optional: Cloud Run

- **Cloud Run Service:** `google_cloud_run_v2_service.adapter`
  - Enabled via `enable_cloud_run_adapter` variable (default: false)
  - min_instances: 0 (scales to zero, no idle cost)
  - max_instances: 1
  - No public invoker permission (requires authentication)
  - Placeholder image (replace with actual adapter when built)

### API Enablement

- `compute.googleapis.com`
- `iap.googleapis.com`
- `run.googleapis.com` (if Cloud Run enabled)

## Variables

See `infra/terraform/variables.tf` for full list. Key variables:

| Variable | Default | Validation |
|----------|---------|------------|
| `project_id` | (required) | Non-empty string |
| `region` | `us-west1` | Must be us-west1, us-central1, or us-east1 |
| `zone` | `us-west1-a` | Any zone in region |
| `machine_type` | `e2-micro` | Must be exactly `e2-micro` |
| `boot_disk_size_gb` | 20 | Between 1 and 30 GB |
| `game_port` | 7777 | Between 1024 and 65535 |
| `network_cidr` | `10.0.0.0/24` | Valid IPv4 CIDR |
| `enable_cloud_run_adapter` | false | Boolean |

## Outputs

See `infra/terraform/outputs.tf`. Notable outputs:

- `external_ip` - Ephemeral external IP (changes on stop/start)
- `ssh_command` - IAP SSH command
- `game_server_address` - IP:PORT for clients
- `estimated_monthly_cost` - "$0.00 (Always Free tier)"
- `cost_warnings` - List of potential charges

## Cost Analysis

### Always Free Tier Limits

**Compute Engine:**
- 1 e2-micro VM/month in us-west1, us-central1, or us-east1
- 30 GB standard persistent disk
- 1 GB snapshot storage
- 1 GB network egress/month from North America

**Cloud Run:**
- 2 million requests/month
- 360,000 GB-seconds memory
- 180,000 vCPU-seconds

### Expected Cost

**Baseline:** $0.00/month (all resources within Always Free tier)

### Potential Charges

1. **Network egress beyond 1 GB/month:** ~$0.12/GB
   - 10 players at 50 KB/s average = 1.8 GB/hour
   - Budget: Keep concurrent users <10 sustained

2. **VM in wrong region:** ~$3-15/month
   - **Prevented by validation:** Region restricted to Always Free zones

3. **Static IP:** ~$2.88/month
   - **Not provisioned:** Only ephemeral IP used

4. **Load balancer:** ~$18/month minimum
   - **Not provisioned:** No forwarding rules or backend services

### Cost Controls

- **Validation:** Variables enforce Always Free constraints
- **No static IP:** Resource not created
- **No LB/MIG:** Resources not created
- **Documentation:** Cost monitoring instructions in ZERO-COST.md
- **Billing alerts:** Instructions for $1 budget alert

## Security

### Network

- Game port (UDP 7777) open to all (required for gameplay)
- SSH only via IAP tunnel (no public key auth, no password auth)
- Optional: Restrict SSH to specific source IPs
- No other ports exposed

### IAM

- VM service account has minimal permissions (logging, monitoring only)
- No storage access, no secrets access
- OS Login enabled (IAM-based SSH, no shared keys)
- Cloud Run (if enabled) requires authentication (no allUsers)

### Secrets

- No secrets in terraform.tfvars.example
- terraform.tfvars gitignored
- terraform.tfstate gitignored (contains sensitive data)
- No API keys or credentials in code

## Verification Steps

### Terraform Validation

```bash
cd infra/terraform
terraform fmt -check -recursive  # Format check
terraform init                    # Initialize providers
terraform validate                # Validate configuration
```

**Result:** All checks passed (2026-09-18)

### Cost Safety Checklist

- [x] Machine type is `e2-micro` with validation
- [x] Region restricted to Always Free zones
- [x] Boot disk <= 30 GB with validation
- [x] No `google_compute_address` (static IP)
- [x] No `google_compute_backend_service` (load balancer)
- [x] No `google_container_cluster` (GKE)
- [x] No `google_compute_instance_group_manager` (MIG)
- [x] Service account has minimal permissions
- [x] .gitignore excludes tfstate and secrets
- [x] terraform.tfvars.example has no secrets

### Documentation Checklist

- [x] infra/README.md with deployment workflow
- [x] infra/docs/ZERO-COST.md with cost boundaries
- [x] infra/terraform/README.md with usage instructions
- [x] terraform fmt/validate instructions
- [x] No terraform apply without approval gate
- [x] Cost monitoring instructions
- [x] Troubleshooting guide

## Success Criteria

1. Terraform configuration validates successfully
2. All resources fit within Always Free tier
3. No static IP, load balancer, MIG, or GKE resources
4. Service account has minimal permissions
5. Documentation includes cost monitoring and approval gates
6. .gitignore excludes tfstate and secrets
7. terraform.tfvars.example has no secrets
8. PR created for Gitty review

**Result:** All criteria met (2026-09-18)

## Risks and Mitigations

### Risk: Accidental Apply

**Impact:** Resources created without approval, potential costs

**Mitigation:**
- Documentation emphasizes approval requirement
- No `terraform apply` instructions in normal workflow
- Draft PR requires Gitty review
- Variables validate Always Free constraints

### Risk: Network Egress Costs

**Impact:** Charges beyond 1 GB/month egress

**Mitigation:**
- ZERO-COST.md documents egress limits
- Instructions for billing alerts
- Expected usage calculation (10 users = 1.8 GB/hour)
- Guidance to stop VM if costs appear

### Risk: Static IP Created by Mistake

**Impact:** ~$2.88/month charge

**Mitigation:**
- No static IP resource in code
- Only ephemeral IP used
- Documentation explains cost difference
- Gitty review should catch if added

### Risk: VM in Wrong Region

**Impact:** ~$3-15/month charge

**Mitigation:**
- Region variable validates against Always Free zones
- Terraform will reject invalid regions
- Default is us-west1 (Always Free)

### Risk: State File Committed

**Impact:** Sensitive data in git history

**Mitigation:**
- .gitignore excludes *.tfstate, *.tfstate.*, .terraform/
- Documentation warns against committing state
- Optional remote state backend documented (GCS)

## Deployment Workflow

### Phase 1: Plan Only (Current)

1. Create Terraform configuration
2. Validate with fmt/validate
3. Create PR for review
4. Merge to main (after Gitty approval)

**Status:** Complete (PR #5)

### Phase 2: Apply (Future, Requires Approval)

1. Nick or Chief approves infrastructure deployment
2. Copy terraform.tfvars.example to terraform.tfvars
3. Set GCP project ID
4. Run `terraform plan` and review
5. Get explicit ACK for apply
6. Run `terraform apply`
7. Note external IP from outputs
8. Set up billing alerts

**Status:** Blocked (awaiting approval)

### Phase 3: Deploy Server (Future)

1. SSH via IAP: `gcloud compute ssh fragr-game-server --zone=us-west1-a --tunnel-through-iap`
2. Copy server binary to VM (scp or GCS)
3. Configure systemd service (optional)
4. Start server: `./fragr-server --bind 0.0.0.0:7777`
5. Test connectivity from client
6. Monitor costs in GCP Console

**Status:** Not started

## Alternatives Considered

### Oracle Cloud Always Free

**Pros:**
- More generous: 2 VMs, 200 GB storage, 10 TB egress/month
- Ampere A1 or E2.1 Micro instances

**Cons:**
- Less familiar than GCP
- Account approval can be slow
- Network quality varies

**Decision:** Defer to future. GCP for initial deployment (familiar, well-documented).

### Fly.io Free Tier

**Pros:**
- 3 shared-cpu VMs, 3 GB persistent storage
- Good developer experience

**Cons:**
- UDP support experimental (game server requires UDP)
- Shared CPU may not be sufficient

**Decision:** Not suitable for Slice 1 (UDP requirement).

### Self-Host Only

**Pros:**
- True $0 (no cloud)
- Full control

**Cons:**
- Port forwarding complexity
- Dynamic IP changes
- Home network limitations

**Decision:** Valid option, documented in README. Infrastructure prepared for when cloud is needed.

## Future Enhancements

### After Slice 1 Apply

- [ ] Systemd service file for server
- [ ] Log aggregation to Cloud Logging
- [ ] Metrics dashboard in Cloud Monitoring
- [ ] Automated deployment scripts
- [ ] Backup and restore procedures

### Optional Features

- [ ] Cloud Build for server binary CI/CD (if Gitty approves)
- [ ] Cloud Run adapter implementation (if needed)
- [ ] Multiple regions (if users outside US)
- [ ] Terraform remote state (GCS backend)
- [ ] Custom domain and DNS (if desired)

### Cost Optimization

- [ ] Network egress monitoring dashboard
- [ ] Auto-shutdown during low usage hours
- [ ] Ephemeral IP caching for stable client connections
- [ ] Alternative free hosts if GCP limits reached

## References

- [GCP Always Free Tier Documentation](https://cloud.google.com/free/docs/free-cloud-features)
- [Terraform GCP Provider](https://registry.terraform.io/providers/hashicorp/google/latest/docs)
- [Identity-Aware Proxy](https://cloud.google.com/iap/docs/using-tcp-forwarding)
- [Compute Engine Pricing](https://cloud.google.com/compute/all-pricing)
- [GCP Free Trial](https://cloud.google.com/free)

## Approval Required

**Before terraform apply:**

- [ ] Nick or Chief reviews terraform plan output
- [ ] Verifies all resources fit Always Free tier
- [ ] Acknowledges potential network egress costs
- [ ] Approves infrastructure deployment

**PR Review (Gitty):**

- [ ] Cost safety verified
- [ ] Security appropriate
- [ ] Documentation complete
- [ ] No secrets in code

---

**Plan written:** 2026-09-18  
**Implementation:** Complete (plan only)  
**PR:** #5 (https://github.com/blisspixel/fragr/pull/5)  
**Status:** Ready for Gitty review

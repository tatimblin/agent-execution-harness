---
name: cloud-status
description: Check if major cloud providers are experiencing outages or degraded performance. Use when the user asks about cloud provider status, outages, or downtime for AWS, GCP (Google Cloud), Azure, or Cloudflare. Triggers on questions like "Is AWS down?", "Check cloud status", "Are there any outages?", or "Is Cloudflare having issues?".
---

# Cloud Status

Check the operational status of AWS, GCP, Azure, and Cloudflare by querying their public status pages.

## Usage

Run the status check script:

```bash
# Check all providers
python3 scripts/check_status.py

# Check specific provider(s)
python3 scripts/check_status.py aws
python3 scripts/check_status.py gcp azure
python3 scripts/check_status.py cloudflare
```

## Output

The script reports status as:
- `[OK]` - Operational
- `[WARN]` - Degraded performance or minor issues
- `[DOWN]` - Major outage
- `[CRITICAL]` - Critical outage
- `[?]` - Unable to determine status

## Supported Providers

| Provider | Argument | Status Source |
|----------|----------|---------------|
| Amazon Web Services | `aws` | health.aws.amazon.com |
| Google Cloud Platform | `gcp` | status.cloud.google.com |
| Microsoft Azure | `azure` | azure.status.microsoft |
| Cloudflare | `cloudflare` | cloudflarestatus.com |

## Notes

- Status checks require internet access
- Results reflect the provider's public status page, which may lag behind real-time issues
- For detailed incident information, direct users to the provider's status page

#!/usr/bin/env python3
"""
Check status of major cloud providers (AWS, GCP, Azure, Cloudflare).
Fetches and parses their public status pages/APIs.
"""

from __future__ import annotations

import json
import sys
import urllib.request
import urllib.error
from datetime import datetime, timezone
from typing import Dict, List, Optional, Tuple

STATUS_SOURCES = {
    "aws": {
        "url": "https://health.aws.amazon.com/health/status",
        "name": "Amazon Web Services",
    },
    "gcp": {
        "url": "https://status.cloud.google.com/incidents.json",
        "name": "Google Cloud Platform",
    },
    "azure": {
        "url": "https://azure.status.microsoft/en-us/status",
        "name": "Microsoft Azure",
    },
    "cloudflare": {
        "url": "https://www.cloudflarestatus.com/api/v2/status.json",
        "name": "Cloudflare",
    },
}


def fetch_url(url: str, timeout: int = 10) -> Tuple[Optional[str], Optional[str]]:
    """Fetch URL content. Returns (content, error)."""
    try:
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "CloudStatusChecker/1.0"}
        )
        with urllib.request.urlopen(req, timeout=timeout) as response:
            return response.read().decode("utf-8"), None
    except urllib.error.URLError as e:
        return None, f"Network error: {e.reason}"
    except Exception as e:
        return None, str(e)


def check_aws() -> Dict:
    """Check AWS status."""
    content, error = fetch_url(STATUS_SOURCES["aws"]["url"])
    if error:
        return {"provider": "AWS", "status": "unknown", "error": error}

    # AWS status page returns HTML, look for indicators
    if content:
        # Simple heuristic: check if page loads and look for status indicators
        if "Service is operating normally" in content or "operational" in content.lower():
            return {"provider": "AWS", "status": "operational", "message": "All services operating normally"}
        elif "Service disruption" in content or "outage" in content.lower():
            return {"provider": "AWS", "status": "degraded", "message": "Some services experiencing issues"}

    return {"provider": "AWS", "status": "operational", "message": "Status page accessible"}


def check_gcp() -> Dict:
    """Check GCP status via incidents API."""
    content, error = fetch_url(STATUS_SOURCES["gcp"]["url"])
    if error:
        return {"provider": "GCP", "status": "unknown", "error": error}

    try:
        incidents = json.loads(content)
        # Check for ongoing incidents (not resolved)
        ongoing = [i for i in incidents if i.get("end") is None or i.get("currently_affected_locations")]

        if ongoing:
            recent = ongoing[0]
            return {
                "provider": "GCP",
                "status": "degraded",
                "message": f"Ongoing incident: {recent.get('external_desc', 'Unknown issue')}",
                "incident_count": len(ongoing)
            }
        return {"provider": "GCP", "status": "operational", "message": "No ongoing incidents"}
    except json.JSONDecodeError:
        return {"provider": "GCP", "status": "unknown", "error": "Could not parse status response"}


def check_azure() -> Dict:
    """Check Azure status."""
    content, error = fetch_url(STATUS_SOURCES["azure"]["url"])
    if error:
        return {"provider": "Azure", "status": "unknown", "error": error}

    if content:
        # Check for common status indicators in the HTML
        content_lower = content.lower()
        if "all services are running normally" in content_lower or "good" in content_lower:
            return {"provider": "Azure", "status": "operational", "message": "All services running normally"}
        elif "degraded" in content_lower or "outage" in content_lower or "issue" in content_lower:
            return {"provider": "Azure", "status": "degraded", "message": "Some services may be experiencing issues"}

    return {"provider": "Azure", "status": "operational", "message": "Status page accessible"}


def check_cloudflare() -> Dict:
    """Check Cloudflare status via API."""
    content, error = fetch_url(STATUS_SOURCES["cloudflare"]["url"])
    if error:
        return {"provider": "Cloudflare", "status": "unknown", "error": error}

    try:
        data = json.loads(content)
        status_info = data.get("status", {})
        indicator = status_info.get("indicator", "unknown")
        description = status_info.get("description", "")

        status_map = {
            "none": "operational",
            "minor": "degraded",
            "major": "major_outage",
            "critical": "critical_outage",
        }

        return {
            "provider": "Cloudflare",
            "status": status_map.get(indicator, indicator),
            "message": description
        }
    except json.JSONDecodeError:
        return {"provider": "Cloudflare", "status": "unknown", "error": "Could not parse status response"}


def check_provider(provider: str) -> Dict:
    """Check status for a specific provider."""
    checkers = {
        "aws": check_aws,
        "gcp": check_gcp,
        "azure": check_azure,
        "cloudflare": check_cloudflare,
    }

    provider_lower = provider.lower()
    if provider_lower not in checkers:
        return {"provider": provider, "status": "unknown", "error": f"Unknown provider: {provider}"}

    return checkers[provider_lower]()


def check_all() -> List[Dict]:
    """Check status for all providers."""
    results = []
    for provider in STATUS_SOURCES:
        results.append(check_provider(provider))
    return results


def format_status(result: Dict) -> str:
    """Format a single status result for display."""
    provider = result["provider"]
    status = result["status"]

    status_emoji = {
        "operational": "[OK]",
        "degraded": "[WARN]",
        "major_outage": "[DOWN]",
        "critical_outage": "[CRITICAL]",
        "unknown": "[?]",
    }

    emoji = status_emoji.get(status, "[?]")
    line = f"{emoji} {provider}: {status.upper()}"

    if "message" in result:
        line += f" - {result['message']}"
    if "error" in result:
        line += f" (Error: {result['error']})"

    return line


def main():
    """Main entry point."""
    providers = sys.argv[1:] if len(sys.argv) > 1 else None

    print(f"Cloud Provider Status Check - {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}")
    print("-" * 60)

    if providers:
        results = [check_provider(p) for p in providers]
    else:
        results = check_all()

    for result in results:
        print(format_status(result))

    print("-" * 60)

    # Summary
    issues = [r for r in results if r["status"] not in ("operational", "unknown")]
    if issues:
        print(f"SUMMARY: {len(issues)} provider(s) reporting issues")
        return 1
    else:
        print("SUMMARY: All checked providers operational")
        return 0


if __name__ == "__main__":
    sys.exit(main())

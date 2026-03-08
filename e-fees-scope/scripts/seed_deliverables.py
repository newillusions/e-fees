#!/usr/bin/env python3
"""
Seed deliverables into the scope service from corpus extraction output.

Reads corpus_deliverables.json (output of extract_deliverables.py) and creates
deliverable records via POST /deliverables for items with count >= threshold.

All seeded items are created as layer=generic, status is set to 'draft' by default
(the API sets status='active', but we PATCH to 'draft' after creation for curation).

Usage:
    python3 seed_deliverables.py [--min-count N] [--dry-run] [--api-url URL]
"""

import argparse
import json
import sys
import urllib.request

DEFAULT_API_URL = "http://10.0.21.81:3201"
DEFAULT_API_KEY = "efees-scope-2026-s7k2m9xp"


def api_post(url, api_key, path, data):
    """POST JSON to the scope API."""
    body = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        f"{url}{path}",
        data=body,
        headers={
            "X-API-Key": api_key,
            "Content-Type": "application/json",
            "Accept": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def make_short_name(canonical_text):
    """
    Derive a short_name from canonical text.
    Truncate to ~60 chars, title-case, strip trailing prepositions.
    """
    # Clean up common artifacts
    text = canonical_text.strip()
    # Remove trailing content in brackets/parens if it makes it too long
    if len(text) > 80:
        for sep in [" (", " [", " -"]:
            idx = text.find(sep)
            if idx > 20:
                text = text[:idx]
                break

    # Truncate to 60 chars at word boundary
    if len(text) > 60:
        text = text[:57].rsplit(" ", 1)[0] + "..."

    return text


def main():
    parser = argparse.ArgumentParser(description="Seed deliverables from corpus extraction")
    parser.add_argument("--input", default="corpus_deliverables.json", help="Input JSON file")
    parser.add_argument("--min-count", type=int, default=5, help="Minimum cluster count to include")
    parser.add_argument("--api-url", default=DEFAULT_API_URL)
    parser.add_argument("--api-key", default=DEFAULT_API_KEY)
    parser.add_argument("--dry-run", action="store_true", help="Print what would be created without sending")
    args = parser.parse_args()

    # Load extraction data
    with open(args.input) as f:
        data = json.load(f)

    clusters_by_stage = data.get("clusters_by_stage", {})

    # Filter to items above threshold
    to_seed = []
    for stage, clusters in clusters_by_stage.items():
        sort_order = 1
        for cluster in clusters:
            if cluster["count"] >= args.min_count:
                to_seed.append({
                    "title": cluster["canonical"],
                    "short_name": make_short_name(cluster["canonical"]),
                    "body": cluster["canonical"],
                    "stage": stage,
                    "layer": "generic",
                    "sort_order": sort_order,
                    "source_proposals": cluster.get("sources", [])[:10],
                    "tags": [f"corpus-count:{cluster['count']}"],
                })
                sort_order += 1

    print(f"Items above threshold (count >= {args.min_count}): {len(to_seed)}", file=sys.stderr)

    # Group summary
    by_stage = {}
    for item in to_seed:
        by_stage.setdefault(item["stage"], []).append(item)

    print("\nPer-stage breakdown:", file=sys.stderr)
    for stage in ["preliminaries", "concept", "schematic", "detailed", "ift", "ifc", "post_contract", "handover"]:
        items = by_stage.get(stage, [])
        if items:
            print(f"  {stage:16s}: {len(items)} items", file=sys.stderr)

    if args.dry_run:
        print(f"\n[DRY RUN] Would create {len(to_seed)} deliverables. Sample:", file=sys.stderr)
        for item in to_seed[:5]:
            print(f"  [{item['stage']}] {item['short_name']}", file=sys.stderr)
        # Output full list as JSON
        print(json.dumps(to_seed, indent=2, ensure_ascii=False))
        return

    # Seed via API
    created = 0
    errors = 0
    for idx, item in enumerate(to_seed):
        try:
            result = api_post(args.api_url, args.api_key, "/deliverables", item)
            created += 1
            if (idx + 1) % 10 == 0:
                print(f"  [{idx + 1}/{len(to_seed)}] created...", file=sys.stderr)
        except Exception as e:
            errors += 1
            print(f"  ERROR creating '{item['short_name']}': {e}", file=sys.stderr)

    print(f"\nDone: {created} created, {errors} errors out of {len(to_seed)} items", file=sys.stderr)


if __name__ == "__main__":
    main()

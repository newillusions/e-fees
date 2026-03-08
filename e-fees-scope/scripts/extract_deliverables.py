#!/usr/bin/env python3
"""
Extract deliverable items from ingested corpus documents.
Parses "Typical deliverables include:" bullet lists per stage from each proposal.
Groups and clusters similar items across proposals for curation.

Usage:
    python3 extract_deliverables.py [--api-url URL] [--api-key KEY] [--output FILE]
"""

import argparse
import json
import re
import sys
from collections import defaultdict
from difflib import SequenceMatcher
import urllib.request

DEFAULT_API_URL = "http://10.0.21.81:3201"
DEFAULT_API_KEY = "efees-scope-2026-s7k2m9xp"

# Canonical stage mapping — maps various stage names to our 8 canonical stages
STAGE_MAP = {
    "preliminaries": "preliminaries",
    "stage 0": "preliminaries",
    "mobilisation": "preliminaries",
    # Concept
    "concept": "concept",
    "concept design": "concept",
    "stage 1": "concept",
    "stage 1 -": "concept",
    "information gathering": "concept",
    # Schematic
    "schematic": "schematic",
    "schematic design": "schematic",
    "stage 2": "schematic",
    "stage 2 -": "schematic",
    "report and recommendations": "schematic",
    # Detailed
    "detailed": "detailed",
    "detailed design": "detailed",
    "stage 3": "detailed",
    "stage 3 -": "detailed",
    "on-going support": "detailed",
    # IFT (Issued for Tender)
    "ift": "ift",
    "tender": "ift",
    "tender documents": "ift",
    "tender documentation": "ift",
    "stage 4": "ift",
    "stage 4 -": "ift",
    "stage 4a": "ift",
    "stage 4a -": "ift",
    # IFC — tender return / evaluation
    "ifc": "ifc",
    "tender return": "ifc",
    "tender return evaluation": "ifc",
    "tender return review": "ifc",
    "stage 4b": "ifc",
    "stage 4b -": "ifc",
    "stage 5": "ifc",
    "stage 5 -": "ifc",
    # Post Contract
    "post contract": "post_contract",
    "post contract phase": "post_contract",
    "construction supervision": "post_contract",
    "stage 6": "post_contract",
    "stage 6 -": "post_contract",
    "focussing": "post_contract",
    "focusing": "post_contract",
    "aiming": "post_contract",
    "scene setting": "post_contract",
    "stage 5a": "post_contract",
    "stage 6a": "post_contract",
    "stage 6a -": "post_contract",
    # Handover
    "handover": "handover",
    "hand over": "handover",
    "close out": "handover",
    "hand over and close out": "handover",
    "stage 6b": "handover",
    "stage 6b -": "handover",
    "stage 7": "handover",
    "stage 7 -": "handover",
    "post completion": "handover",
    "dlp": "handover",
}


def api_request(url, api_key, path):
    """Make authenticated GET request to scope API."""
    req = urllib.request.Request(
        f"{url}{path}",
        headers={"X-API-Key": api_key, "Accept": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())


def normalize_stage(header_text):
    """Map a stage header to a canonical stage name."""
    text = header_text.lower().strip()
    # Remove markdown bold, brackets, numbering artifacts
    text = re.sub(r"\*\*", "", text)
    text = re.sub(r"\[riba\]", "", text, flags=re.IGNORECASE)
    text = text.strip(" -*#")

    # Try exact matches first, then prefix matches
    for pattern, canonical in STAGE_MAP.items():
        if text == pattern or text.startswith(pattern):
            return canonical

    # Fuzzy fallback
    for pattern, canonical in STAGE_MAP.items():
        if pattern in text:
            return canonical

    return None


def extract_deliverables_from_text(text, filename):
    """
    Parse a corpus document's extracted text for deliverable bullet lists.
    Returns list of {stage, items: [str], source_context: str}
    """
    lines = text.split("\n")
    results = []
    current_stage = None
    current_stage_raw = None

    i = 0
    while i < len(lines):
        line = lines[i].strip()
        lower = line.lower()

        # Skip page markers
        if re.match(r"\[PAGE \d+\]", line):
            i += 1
            continue

        # Detect stage headers
        stage_match = normalize_stage(line)
        if stage_match and len(line) < 100:  # stage headers are short
            current_stage = stage_match
            current_stage_raw = line
            i += 1
            continue

        # Detect deliverable list markers
        if re.match(
            r"(typical|specified|key)\s+deliverables?\s+include", lower
        ):
            if current_stage is None:
                # Try to detect stage from nearby context
                for back in range(max(0, i - 10), i):
                    candidate = normalize_stage(lines[back].strip())
                    if candidate:
                        current_stage = candidate
                        current_stage_raw = lines[back].strip()
                        break

            if current_stage is None:
                current_stage = "unknown"
                current_stage_raw = "Unknown"

            # Collect bullet items
            items = []
            j = i + 1
            while j < len(lines):
                bullet_line = lines[j].strip()

                # End conditions
                if not bullet_line:
                    j += 1
                    # Allow one blank line between bullets
                    if j < len(lines) and lines[j].strip().startswith("- "):
                        continue
                    # Check if next non-blank line is also a bullet
                    while j < len(lines) and not lines[j].strip():
                        j += 1
                    if j < len(lines) and lines[j].strip().startswith("- "):
                        continue
                    break

                if bullet_line.startswith("- "):
                    items.append(bullet_line[2:].strip())
                elif re.match(r"(stage|---|\[page|important|note|p\.\d)", bullet_line.lower()):
                    break
                elif re.match(r"(typical|specified|key)\s+deliverables", bullet_line.lower()):
                    break
                elif items:
                    # Non-bullet continuation line — might be part of previous item
                    # Only if previous line wasn't empty
                    if items and not bullet_line.startswith("|"):
                        items[-1] += " " + bullet_line
                j += 1

            if items:
                results.append(
                    {
                        "stage": current_stage,
                        "stage_raw": current_stage_raw,
                        "items": items,
                        "source": filename,
                    }
                )

            i = j
            continue

        i += 1

    return results


def similarity(a, b):
    """Compute string similarity ratio."""
    return SequenceMatcher(None, a.lower(), b.lower()).ratio()


def cluster_items(all_items):
    """
    Cluster similar deliverable items across proposals.
    Returns list of clusters: {canonical: str, variants: [{text, source, stage}], count: int}
    """
    # Group by stage first
    by_stage = defaultdict(list)
    for item in all_items:
        by_stage[item["stage"]].append(item)

    all_clusters = {}
    for stage, items in by_stage.items():
        clusters = []
        for item in items:
            text = item["text"]
            # Find best matching cluster
            best_match = None
            best_score = 0
            for cluster in clusters:
                score = similarity(text, cluster["canonical"])
                if score > best_score:
                    best_score = score
                    best_match = cluster

            if best_match and best_score >= 0.65:
                best_match["variants"].append(
                    {"text": text, "source": item["source"]}
                )
                best_match["count"] += 1
                # Use the longest variant as canonical
                if len(text) > len(best_match["canonical"]):
                    best_match["canonical"] = text
            else:
                clusters.append(
                    {
                        "canonical": text,
                        "variants": [{"text": text, "source": item["source"]}],
                        "count": 1,
                        "stage": stage,
                    }
                )

        for c in clusters:
            key = f"{stage}::{c['canonical'][:60]}"
            all_clusters[key] = c

    return list(all_clusters.values())


def main():
    parser = argparse.ArgumentParser(description="Extract deliverables from corpus")
    parser.add_argument("--api-url", default=DEFAULT_API_URL)
    parser.add_argument("--api-key", default=DEFAULT_API_KEY)
    parser.add_argument("--output", default="-", help="Output file (- for stdout)")
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    # Step 1: List all corpus documents
    print("Fetching corpus list...", file=sys.stderr)
    corpus_list = api_request(args.api_url, args.api_key, "/corpus?limit=100")
    docs = corpus_list.get("data", [])
    print(f"Found {len(docs)} corpus documents", file=sys.stderr)

    # Step 2: Extract deliverables from each document
    all_items = []
    all_extractions = []

    for idx, doc in enumerate(docs):
        doc_id = doc["id"]
        filename = doc["filename"]

        if args.verbose:
            print(
                f"  [{idx+1}/{len(docs)}] Processing {filename}...", file=sys.stderr
            )

        try:
            detail = api_request(args.api_url, args.api_key, f"/corpus/{doc_id}")
            data = detail.get("data", detail)
            if isinstance(data, list):
                data = data[0] if data else {}
            text = data.get("extracted_text", "")

            if not text:
                if args.verbose:
                    print(f"    Skipped (no text)", file=sys.stderr)
                continue

            extractions = extract_deliverables_from_text(text, filename)
            all_extractions.extend(extractions)

            for ext in extractions:
                for item_text in ext["items"]:
                    all_items.append(
                        {
                            "text": item_text,
                            "stage": ext["stage"],
                            "source": filename,
                        }
                    )

            if args.verbose:
                total_items = sum(len(e["items"]) for e in extractions)
                stages = set(e["stage"] for e in extractions)
                print(
                    f"    Found {total_items} items across {len(stages)} stages",
                    file=sys.stderr,
                )

        except Exception as e:
            print(f"    ERROR: {e}", file=sys.stderr)

    # Step 3: Cluster similar items
    print(f"\nTotal raw items: {len(all_items)}", file=sys.stderr)
    clusters = cluster_items(all_items)
    clusters.sort(key=lambda c: (-c["count"], c["stage"], c["canonical"]))
    print(f"Clustered into {len(clusters)} unique deliverables", file=sys.stderr)

    # Step 4: Build output
    # Group clusters by stage
    by_stage = defaultdict(list)
    for cluster in clusters:
        by_stage[cluster["stage"]].append(cluster)

    output = {
        "summary": {
            "total_documents": len(docs),
            "total_raw_items": len(all_items),
            "total_clusters": len(clusters),
            "stages": {
                stage: len(items) for stage, items in sorted(by_stage.items())
            },
        },
        "clusters_by_stage": {},
    }

    stage_order = [
        "preliminaries",
        "concept",
        "schematic",
        "detailed",
        "ift",
        "ifc",
        "post_contract",
        "handover",
        "unknown",
    ]

    for stage in stage_order:
        if stage not in by_stage:
            continue
        stage_clusters = sorted(
            by_stage[stage], key=lambda c: -c["count"]
        )
        output["clusters_by_stage"][stage] = [
            {
                "canonical": c["canonical"],
                "count": c["count"],
                "sources": list(set(v["source"] for v in c["variants"])),
                "variants": [
                    v["text"]
                    for v in c["variants"]
                    if v["text"] != c["canonical"]
                ][:5],  # max 5 variants
            }
            for c in stage_clusters
        ]

    # Write output
    result = json.dumps(output, indent=2, ensure_ascii=False)
    if args.output == "-":
        print(result)
    else:
        with open(args.output, "w") as f:
            f.write(result)
        print(f"\nOutput written to {args.output}", file=sys.stderr)

    # Print summary
    print("\n=== EXTRACTION SUMMARY ===", file=sys.stderr)
    for stage in stage_order:
        if stage in by_stage:
            items = by_stage[stage]
            most_common = max(items, key=lambda c: c["count"])
            print(
                f"  {stage:16s}: {len(items):3d} unique items "
                f"(most common: \"{most_common['canonical'][:60]}\" x{most_common['count']})",
                file=sys.stderr,
            )


if __name__ == "__main__":
    main()

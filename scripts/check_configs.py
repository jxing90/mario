#!/usr/bin/env python3
"""
Project-specific config checker for Mario 2D Platformer Demo.

Reads feature-list.json > required_configs[], checks that all
declared configuration items are present.

- env type: checks os.environ for the key
- file type: checks os.path.exists for the path

Usage:
    python scripts/check_configs.py feature-list.json
    python scripts/check_configs.py feature-list.json --feature <id>

Exit 0: all required configs present (or no configs required)
Exit 1: one or more required configs missing
"""

import argparse
import json
import os
import sys


def load_feature_list(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def check_config(config: dict) -> list[str]:
    """Check a single config entry. Returns list of error strings (empty = OK)."""
    errors = []
    name = config.get("name", "?")
    ctype = config.get("type", "?")
    hint = config.get("check_hint", "No hint provided")

    if ctype == "env":
        key = config.get("key", "")
        if not key:
            errors.append(f"Config '{name}': missing 'key' field for env type")
        elif key not in os.environ or not os.environ[key]:
            errors.append(f"Config '{name}': env var '{key}' is not set or empty. Hint: {hint}")
    elif ctype == "file":
        path = config.get("path", "")
        if not path:
            errors.append(f"Config '{name}': missing 'path' field for file type")
        elif not os.path.exists(path):
            errors.append(f"Config '{name}': file '{path}' does not exist. Hint: {hint}")
    else:
        errors.append(f"Config '{name}': unknown type '{ctype}' (expected 'env' or 'file')")

    return errors


def main():
    parser = argparse.ArgumentParser(
        description="Check required project configurations"
    )
    parser.add_argument(
        "feature_list",
        help="Path to feature-list.json"
    )
    parser.add_argument(
        "--feature", type=int, default=None,
        help="Feature ID to filter configs by (optional)"
    )
    args = parser.parse_args()

    data = load_feature_list(args.feature_list)
    required_configs = data.get("required_configs", [])

    if not required_configs:
        print("No required_configs declared — nothing to check.")
        sys.exit(0)

    # Filter by feature if requested
    if args.feature is not None:
        required_configs = [
            c for c in required_configs
            if args.feature in c.get("required_by", [])
        ]
        if not required_configs:
            print(f"No configs required for feature #{args.feature}.")
            sys.exit(0)

    all_errors = []
    for config in required_configs:
        errors = check_config(config)
        all_errors.extend(errors)

    if all_errors:
        print(f"CONFIG CHECK FAILED — {len(all_errors)} issue(s):\n")
        for err in all_errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        checked = len(required_configs)
        print(f"CONFIG CHECK PASSED — all {checked} required config(s) present.")
        sys.exit(0)


if __name__ == "__main__":
    main()

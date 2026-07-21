"""Small, dependency-free readers for Cargo.lock package records."""

from __future__ import annotations

import json
from pathlib import Path


def package_versions(lock_path: Path, package_name: str) -> list[str]:
    """Return the distinct resolved versions for one Cargo package."""

    versions: set[str] = set()
    package_records = lock_path.read_text(encoding="utf-8").split("[[package]]")[1:]
    for package_record in package_records:
        fields: dict[str, str] = {}
        for line in package_record.splitlines():
            key, separator, value = line.partition("=")
            key = key.strip()
            if not separator or key not in {"name", "version"}:
                continue
            try:
                parsed = json.loads(value.strip())
            except json.JSONDecodeError:
                continue
            if isinstance(parsed, str):
                fields[key] = parsed
        if fields.get("name") == package_name and "version" in fields:
            versions.add(fields["version"])
    return sorted(versions)

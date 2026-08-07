#!/usr/bin/env python3
"""Build deterministic, secret-free PF Terminal release preflight evidence."""

from __future__ import annotations

import csv
import hashlib
import json
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "qa" / "release" / "0.1.27" / "preflight-20260731"
RECOVERY_REF = "refs/pfterminal/recovery/pre-0.1.27-convergence-20260731"
CONVERGENCE_REF = "45a60f03d"
DISPOSITION_FIELDS = [
    "status",
    "path",
    "area",
    "disposition",
    "review_state",
    "rationale",
]


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    ).stdout.strip()


def git_optional(*args: str) -> str | None:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def command(*args: str) -> str:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    ).stdout.strip()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def status_rows() -> list[tuple[str, str]]:
    raw = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    fields = raw.decode("utf-8", errors="strict").split("\0")
    rows: list[tuple[str, str]] = []
    index = 0
    while index < len(fields) and fields[index]:
        entry = fields[index]
        status = entry[:2]
        path = entry[3:]
        rows.append((status, path))
        index += 1
        if "R" in status or "C" in status:
            index += 1
    return rows


def committed_rows() -> list[tuple[str, str]]:
    """Return paths committed after the frozen convergence merge.

    Before RC freeze the inventory is driven by the worktree. After freeze the
    same paths must remain reproducible even though ``git status`` is clean.
    """

    output = git("diff", "--name-status", "--find-renames", f"{CONVERGENCE_REF}..HEAD")
    rows: list[tuple[str, str]] = []
    for line in output.splitlines():
        fields = line.split("\t")
        status = fields[0]
        path = fields[-1]
        rows.append((status, path))
    return rows


def inventory_rows() -> list[tuple[str, str]]:
    rows = {path: status for status, path in committed_rows()}
    rows.update({path: status for status, path in status_rows()})
    return [(rows[path], path) for path in sorted(rows)]


def previous_review_states() -> dict[str, str]:
    path = OUTPUT / "DISPOSITION.csv"
    if not path.is_file():
        return {}
    with path.open(newline="", encoding="utf-8") as source:
        return {
            row["path"]: row["review_state"]
            for row in csv.DictReader(source)
            if row.get("review_state") in {"reviewed", "blocked"}
        }


def classify(status: str, path: str) -> tuple[str, str, str]:
    if path == "FORK_POLICY.md" or path.startswith("docs/current-sprint/"):
        return (
            "release_documentation",
            "retain",
            "Release policy, incident evidence, or execution specification.",
        )
    if (
        "/schema/" in path
        or "/snapshots/" in path
        or path.endswith(".snap")
        or path in {"codex-rs/Cargo.lock", "MODULE.bazel.lock"}
    ):
        return (
            "generated_artifact",
            "regenerate_and_review",
            "Generated output must match the frozen semantic source and have no pending drift.",
        )
    if "D" in status and path.startswith("codex-rs/core/"):
        return (
            "upstream_runtime_convergence",
            "verify_upstream_replacement",
            "Deleted PF runtime code is acceptable only when pinned upstream owns the behavior.",
        )
    if "D" in status and path.startswith("codex-rs/tui/"):
        return (
            "released_tui_surface",
            "verify_or_restore_product_surface",
            "Deleted TUI code requires explicit proof of an upstream replacement or a restored PF surface.",
        )
    if path.startswith("codex-rs/gpu-market/"):
        return (
            "gpu_rental",
            "retain_and_qualify",
            "GPU rental and the sole DeepSeek 0731 recipe are required PF product scope.",
        )
    if path.startswith(("codex-rs/wallet/", "codex-rs/wallet-daemon/")):
        return (
            "wallet",
            "retain_and_qualify",
            "The released wallet and plan-purchase workflow must remain available.",
        )
    if path.startswith("codex-rs/vault/"):
        return (
            "vault",
            "retain_and_qualify",
            "Encrypted provider credentials are required PF product scope.",
        )
    if path.startswith("codex-rs/telegram/"):
        return (
            "telegram",
            "retain_and_qualify",
            "Telegram control and sandbox propagation are required PF product scope.",
        )
    if path.startswith("codex-rs/tui/"):
        return (
            "tui_convergence",
            "semantic_review_required",
            "Confirm upstream terminal behavior and every retained PF command/surface.",
        )
    if path.startswith("codex-rs/core/"):
        return (
            "core_convergence",
            "semantic_review_required",
            "Confirm upstream ownership or document the PF provider/orchestration boundary.",
        )
    if path.startswith("codex-rs/"):
        area = path.split("/", 2)[1]
        return (
            area,
            "semantic_review_required",
            "Changed crate must receive a release disposition and targeted qualification.",
        )
    return (
        "repository",
        "semantic_review_required",
        "Repository-level change must be reconciled before release freeze.",
    )


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    rows = inventory_rows()
    review_states = previous_review_states()
    dispositions = []
    for status, path in rows:
        area, disposition, rationale = classify(status, path)
        dispositions.append(
            {
                "status": status,
                "path": path,
                "area": area,
                "disposition": disposition,
                "review_state": review_states.get(path, "pending"),
                "rationale": rationale,
            }
        )

    with (OUTPUT / "DISPOSITION.csv").open("w", newline="", encoding="utf-8") as target:
        writer = csv.DictWriter(target, fieldnames=DISPOSITION_FIELDS)
        writer.writeheader()
        writer.writerows(dispositions)

    migration_hashes = {}
    for path in sorted((ROOT / "codex-rs" / "state" / "migrations").glob("*.sql")):
        migration_hashes[str(path.relative_to(ROOT))] = sha256(path)

    recovery_commit = git("rev-parse", RECOVERY_REF)
    recovery_tree = git("rev-parse", f"{RECOVERY_REF}^{{tree}}")
    observed_upstream_head = git_optional("rev-parse", "refs/remotes/openai/main")
    binary = ROOT / "codex-rs" / "target" / "debug" / "pfterminal"
    source = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "branch": git("branch", "--show-current"),
        "head": git("rev-parse", "HEAD"),
        "upstream_codex": "413492cd6c3a4d4f8dff6f406247ccda5a9d88aa",
        "observed_upstream_head": observed_upstream_head,
        "observed_upstream_commits_after_pin": (
            int(
                git(
                    "rev-list",
                    "--count",
                    "413492cd6c3a4d4f8dff6f406247ccda5a9d88aa.."
                    f"{observed_upstream_head}",
                )
            )
            if observed_upstream_head is not None
            else None
        ),
        "rollback_tag": "pfterminal-v0.1.26-pre-convergence",
        "recovery_ref": RECOVERY_REF,
        "recovery_commit": recovery_commit,
        "recovery_tree": recovery_tree,
        "status_path_count": len(rows),
        "status_counts": dict(sorted(Counter(status for status, _ in rows).items())),
        "disposition_counts": dict(
            sorted(Counter(row["disposition"] for row in dispositions).items())
        ),
        "rustc": command("rustc", "--version"),
        "cargo": command("cargo", "--version"),
        "cargo_lock_sha256": sha256(ROOT / "codex-rs" / "Cargo.lock"),
        "bazel_lock_sha256": sha256(ROOT / "MODULE.bazel.lock"),
        "debug_binary": {
            "path": str(binary.relative_to(ROOT)),
            "exists": binary.is_file(),
            "sha256": sha256(binary) if binary.is_file() else None,
        },
        "migration_sha256": migration_hashes,
    }
    (OUTPUT / "SOURCE.json").write_text(
        json.dumps(source, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()

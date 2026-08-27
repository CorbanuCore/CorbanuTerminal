"""Small fail-closed evidence primitives, not a runtime authorization engine."""

import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath

FIXTURES = Path("qa/security-levels/fixtures")
CATALOG = FIXTURES / "catalog-v1.json"
PINS = FIXTURES / "pins-v1.json"
ADAPTERS = Path("qa/security-levels/sprints/PF-27-S01/adapter-conformance.json")
MAX_BYTES = 8 * 1024 * 1024
STATUSES = {"pending", "unavailable", "failed", "passed"}
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
HEX40 = re.compile(r"[0-9a-f]{40}\Z")


class EvidenceError(ValueError):
    """Invalid, incomplete or mismatched evidence; never an implicit pass."""


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_bytes(path):
    with path.open("rb") as stream:
        data = stream.read(MAX_BYTES + 1)
    require(len(data) <= MAX_BYTES, "artifact exceeds evidence size limit")
    return data


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON field")
        result[key] = value
    return result


def load_json(path):
    value = json.loads(read_bytes(path), object_pairs_hook=_unique_object)
    require(isinstance(value, dict), "expected a JSON object")
    return value


def write_json(path, value):
    # Exclusive creation keeps historical evidence and concurrent runs intact.
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")


def local_path(root, relative):
    require(isinstance(relative, str) and bool(relative), "missing artifact path")
    parts = PurePosixPath(relative)
    require(
        not parts.is_absolute()
        and ".." not in parts.parts
        and "\\" not in relative
        and ":" not in relative,
        "unsafe artifact path",
    )
    path = (root / relative).resolve()
    require(
        path.is_relative_to(root.resolve()) and path.is_file(),
        "artifact missing or outside evidence root",
    )
    return path


def indexed(rows, label):
    require(isinstance(rows, list) and bool(rows), f"missing {label}")
    result = {}
    for row in rows:
        require(isinstance(row, dict), f"invalid {label} row")
        key = row.get("id")
        require(
            isinstance(key, str) and re.fullmatch(r"[a-z0-9][a-z0-9.-]*", key),
            f"invalid {label} id",
        )
        require(key not in result, f"duplicate {label} id")
        result[key] = row
    return result


def validate_pins(root):
    pins = load_json(root / PINS)
    require(pins.get("schema_version") == 1, "unsupported pins schema")
    for relative, expected in pins["files"].items():
        require(
            digest(read_bytes(local_path(root, relative))) == expected,
            "frozen PF-21/PF-27 artifact changed",
        )
    return pins


def load_catalog(root):
    validate_pins(root)
    catalog = load_json(root / CATALOG)
    require(catalog.get("schema_version") == 1, "unsupported catalog schema")
    ingresses = indexed(catalog.get("ingresses"), "ingresses")
    require(
        set(ingresses)
        == {
            "file",
            "web",
            "document",
            "tool",
            "mcp",
            "connector",
            "email",
            "memory",
            "delegated",
            "unknown",
        },
        "PF-27 SourceKind inventory is incomplete",
    )
    adapters = indexed(load_json(root / ADAPTERS)["fixtures"], "PF-27 adapters")
    attacks = indexed(catalog.get("attacks"), "attacks")
    controls = indexed(catalog.get("controls"), "controls")
    require(
        catalog.get("levels") == ["moderate", "aggressive"],
        "stronger assertions must not redefine Permissive",
    )
    require(
        len(catalog["sinks"]) == len(set(catalog["sinks"])) and catalog["sinks"],
        "sink inventory must be nonempty and unique",
    )
    for row in ingresses.values():
        require(
            row.get("owner")
            and row.get("support") in {"pending", "pending-deny"}
            and row.get("fixture") in adapters,
            "ingress lacks owner/support/fixture",
        )
        local_path(root, row["adapter"])
    covered_attacks, covered_adapters = set(), set()
    for row in controls.values():
        require(row.get("owner") and row.get("behavior"), "control lacks ownership")
        local_path(root, row["boundary"])
        require(
            row.get("attacks") and set(row["attacks"]) <= attacks.keys(),
            "control lacks known attacks",
        )
        require(
            row.get("adapters") and set(row["adapters"]) <= adapters.keys(),
            "control lacks PF-27 adapter contract",
        )
        covered_attacks.update(row["attacks"])
        covered_adapters.update(row["adapters"])
    require(
        covered_attacks == attacks.keys() and covered_adapters == adapters.keys(),
        "unmapped attack or PF-27 adapter",
    )
    for row in attacks.values():
        require(
            row.get("sources") and set(row["sources"]) <= ingresses.keys() | {"*"},
            "attack has unknown ingress",
        )
        require(
            row.get("payload") and isinstance(row.get("facts"), dict) and row["facts"],
            "attack lacks executable expectations",
        )
    return catalog


def candidate_identity(candidate):
    require(
        isinstance(candidate, dict)
        and set(candidate) == {"source_commit", "binary_sha256", "platform"},
        "incomplete candidate identity",
    )
    require(
        isinstance(candidate["source_commit"], str)
        and HEX40.fullmatch(candidate["source_commit"]),
        "invalid candidate commit",
    )
    require(
        isinstance(candidate["binary_sha256"], str)
        and HEX64.fullmatch(candidate["binary_sha256"]),
        "invalid candidate binary digest",
    )
    require(
        candidate["platform"] in {"linux", "macos", "windows"}, "unsupported platform"
    )
    return candidate


def timestamp(value):
    require(isinstance(value, str) and value.endswith("Z"), "timestamp must be UTC Z")
    try:
        return datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError as error:
        raise EvidenceError("invalid evidence timestamp") from error


def validate_run(run, candidate, catalog_sha256, not_before):
    require(
        run.get("schema_version") == 1 and run.get("phase") == "qualification",
        "synthetic/unknown evidence is not product qualification",
    )
    require(
        candidate_identity(run.get("candidate")) == candidate,
        "mixed-candidate evidence",
    )
    require(run.get("catalog_sha256") == catalog_sha256, "stale fixture catalog")
    require(
        isinstance(run.get("run_id"), str)
        and re.fullmatch(r"[0-9a-f]{32}", run["run_id"]),
        "invalid run identity",
    )
    recorded = timestamp(run.get("recorded_at"))
    require(
        timestamp(not_before) <= recorded <= datetime.now(timezone.utc),
        "stale or future evidence",
    )


def checked_artifact(root, reference):
    require(
        isinstance(reference, dict) and set(reference) == {"path", "sha256"},
        "invalid artifact reference",
    )
    data = read_bytes(local_path(root, reference["path"]))
    require(digest(data) == reference["sha256"], "artifact digest mismatch")
    return data

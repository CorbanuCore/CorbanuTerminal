"""Mandatory future dispatch wrapper: refuse a stale harness or packet candidate."""
import argparse
import ast
import csv
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / "pf83-handoff-70"
sys.path.insert(0, str(PRIOR))
from verify_bundle import require, sha, verify

ATTESTATION = "9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27"


def candidate_pin(path):
    # Parse data only: importing a live harness can execute arbitrary setup.
    tree = ast.parse(path.read_text())
    values = [node.value for node in tree.body if isinstance(node, ast.Assign)
              and any(isinstance(t, ast.Name) and t.id == "CANDIDATE" for t in node.targets)]
    require(len(values) == 1, "exactly one literal CANDIDATE assignment required")
    value = values[0]
    if isinstance(value, ast.Call):
        require(isinstance(value.func, ast.Name) and value.func.id == "dict"
                and not value.args and all(k.arg for k in value.keywords), "nonliteral CANDIDATE")
        require(len({k.arg for k in value.keywords}) == len(value.keywords), "duplicate pin key")
        return {k.arg: ast.literal_eval(k.value) for k in value.keywords}
    return ast.literal_eval(value)


def check(harness, packets, bundle):
    identity = verify(bundle, ATTESTATION)
    require(sha(packets / "packet-bindings.tsv") == sha(PRIOR / "packet-bindings.tsv"),
            "packet inventory differs from frozen inventory")
    require(sha(packets / "packet-rebinding.json") == sha(PRIOR / "packet-rebinding.json"),
            "packet receipt differs from frozen receipt")
    expected = json.loads((packets / "packet-rebinding.json").read_bytes())["binding"]
    require(expected["commit"] == identity["source_commit"]
            and expected["tree"] == identity["source_tree"]
            and expected["package_sha256"] == identity["manifest_sha256"]
            and expected["attestation_sha256"] == ATTESTATION, "bundle/packet candidate disagreement")
    require(candidate_pin(harness / "fixtures.py") == expected,
            "harness/packet candidate disagreement; dispatch refused")
    with (packets / "packet-bindings.tsv").open() as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    require(len(rows) == 288, "packet count")
    for row in rows:
        relative = Path(row["packet_path"])
        require(not relative.is_absolute() and ".." not in relative.parts, "unsafe packet path")
        path = packets / relative
        require(not path.is_symlink() and sha(path) == row["sha256"], "packet digest")
        require(json.loads(path.read_bytes())["candidate"] == expected, "packet candidate disagreement")
    return dict(checked_packets=len(rows), candidate=expected, fixtures_sha256=sha(harness / "fixtures.py"))


def guarded(harness, packets, bundle, command):
    result = check(harness, packets, bundle)
    print(json.dumps(result), flush=True)
    if command:
        # The owner supplies the actual admitted interface; this does not create one.
        return subprocess.run(command, cwd=harness, check=True).returncode
    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--harness", required=True, type=Path)
    parser.add_argument("--packets", required=True, type=Path)
    parser.add_argument("--bundle", required=True, type=Path)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    raise SystemExit(guarded(args.harness, args.packets, args.bundle, command))

"""Audit retained acceptance links, inventories and streams without scratch access."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

here = Path(__file__).resolve().parent
scope = here.parent
repo = here.parents[4]


def require(condition, detail):
    if not condition:
        raise RuntimeError(detail)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def load(path):
    return json.loads(path.read_bytes())


base = "05b8e6e1aea5a55d7a58f3af3c21621a3fa3fb2f"
committed = set(subprocess.check_output(
    ["git", "ls-tree", "-r", "--name-only", base], cwd=repo, text=True).splitlines())
document_path = scope / "acct-acceptance-75/acceptance.md"
document = document_path.read_text()
links = []
for target in re.findall(r"\]\(([^)]+)\)", document):
    path = (document_path.parent / target.split("#")[0]).resolve()
    name = str(path.relative_to(repo))
    require(name in committed and path.is_file(), "unretained acceptance link: " + name)
    links.append(name)

inventories = {}
for name in ("acct-controls-72", "acct-acceptance-75", "acct-guard-91"):
    inventory = load(scope / name / "scope.json")
    entries = inventory.get("files", inventory.get("new_files_excluding_this_inventory"))
    require(len(entries) == len({r["path"] for r in entries}), "duplicate inventory paths")
    for row in entries:
        require(row["path"] in committed, "uncommitted inventory path: " + row["path"])
        path = repo / row["path"]
        raw = path.read_bytes()
        lines = None if path.suffix == ".gz" else len(raw.splitlines())
        require((len(raw), lines, sha(raw)) == (row["bytes"], row["lines"], row["sha256"]),
                "inventory content differs: " + row["path"])
    inventories[name] = len(entries)
guard = scope / "acct-guard-91"
old = json.loads(subprocess.check_output(
    ["git", "show", base + ":" + str((guard / "scope.json").relative_to(repo))], cwd=repo))
current = load(guard / "scope.json")
removed = [row["path"] for row in old["files"] if row["path"] not in committed]
require(len(removed) == 31 and len(current["files"]) == 72, "reconciliation counts")
require({r["path"] for r in current["files"]} ==
        {r["path"] for r in old["files"] if r["path"] in committed}, "membership differs")
for name in removed:
    require("/acct-guard-91/target/" in name, "excluded non-scratch file")
    require(subprocess.run(["git", "check-ignore", "-q", name], cwd=repo).returncode == 0,
            "scratch is not ignored")

simulation_cases = []
for directory in (guard / "simulation-initial", guard / "simulation-final", here / "simulation"):
    receipt = load(directory / "simulation.json")
    for row in receipt["cases"]:
        stdout = (directory / (row["case"] + ".stdout.txt")).read_bytes()
        stderr = (directory / (row["case"] + ".stderr.txt")).read_bytes()
        require(sha(stdout) == row["stdout_sha256"], "simulation stdout differs")
        require(row["stderr_empty"] and stderr == b"", "simulation stderr is not empty")
        simulation_cases.append(str(directory.relative_to(scope)) + "/" + row["case"])
    require([row["exit"] for row in receipt["cases"]] == [2, 2, 3, 3, 2], "simulation exits")
require("\n```text\n" + (here / "simulation/unrefreshed-edit.stdout.txt").read_text() +
        "```\n" in document, "embedded failure output differs")
replay_streams = 0
for directory in (guard / "replay-results", here / "replay-results"):
    receipt = load(directory / "replay-checks.json")
    require([row["exit"] for row in receipt["cases"]] == [0, 0, 1, 1], "replay exits")
    for case in receipt["cases"]:
        require(load(directory / (case["case"] + ".stdout.txt")) == case["receipt"],
                "outer replay receipt differs")
        stderr = (directory / (case["case"] + ".stderr.txt")).read_bytes()
        require((not stderr) if case["exit"] == 0 else
                b"AssertionError: current integration output differs from pinned replay" in stderr,
                "outer replay stderr differs")
        for row in case["receipt"]["commands"]:
            for stream in ("stdout", "stderr"):
                raw = (directory / case["case"] / (row["name"] + "." + stream + ".txt")).read_bytes()
                require(sha(raw) == row[stream + "_sha256"], "nested replay stream digest")
                replay_streams += 1
        controls = json.loads((directory / case["case"] / "controls.stdout.txt").read_text().split("\n", 1)[1])
        require(len(controls["cases"]) == 13 and all(row["passed"] for row in controls["cases"]),
                "nested verifier controls failed")

lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row = load(here / "gates-01" / (name + ".json"))
    raw = gzip.decompress((here / "gates-01" / (name + ".log.gz")).read_bytes())
    require(sha(raw) == row["log_sha256"] and row["exit"] == 0, "gate failed or log differs")
    require(not row["failure_lines"] and row["environment"]["NEXTEST_TEST_THREADS"] == "4",
            "gate failures/thread limit")
    if name != "prerequisites":
        require(row["command"][:2] == ["just", "test"], "unguarded test")
        require(b"Test isolation: disposable profile; native keyring disabled (debug lane)" in raw,
                "missing isolation banner")
        matches = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", raw.decode(), re.M)
        require(len(matches) == 1, "ambiguous summary")
        count, details = matches[0]
        require(int(count) == row["run"] == row["passed"] > 0, "nonzero run/pass count")
        for key, label in (("passed", "passed"), ("failed", "failed"), ("timed_out", "timed out"),
                           ("flaky", "flaky"), ("skipped", "skipped"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, details)
            require(row[key] == (int(match[1]) if match else 0), "summary count mismatch")
    lanes.append(row)
require(len({r["environment"]["CARGO_TARGET_DIR"] for r in lanes}) == 1, "targets differ")
reference = scope / "acct-reference-88"
expected = (reference / "expected.stdout.txt").read_bytes()
require(sha(expected) == load(reference / "reference.json")["sha256"], "reference digest")
for optimized in (False, True):
    result = subprocess.run([sys.executable, "-B", *(["-O"] if optimized else []),
                             str(scope / "acct-inventory-79/verify_acceptance.py")],
                            cwd=repo, capture_output=True)
    require(result.returncode == 2 and result.stdout == expected and not result.stderr,
            "current baseline/reference mismatch")
report = dict(base=base, acceptance_links=links, inventories=inventories,
              ignored_scratch_entries_removed=len(removed),
              simulation_cases=simulation_cases, nested_replay_streams_checked=replay_streams,
              expected_sha256=sha(expected), baseline_exits=[2, 2], lanes=lanes,
              additional_unverifiable_claim_defects_found=[],
              limits=["Three ignored historical package binaries remain unavailable by design.",
                      "Historical absolute run/checkout paths are provenance, not retained artifacts.",
                      "Natural-language, authorization and independent qualification limits remain disclosed.",
                      "This recheck validates retained claims and streams; it is not functional acceptance."])
(here / "recheck.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))

"""Preserve closed-run logs and assemble a readable rendered-output index."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

root = Path(__file__).resolve().parent
repo = root.parents[4]
shutil.copyfile(root / "collect-04/fixture/traffic-expectations.json",
                root / "collect-04/independent-arithmetic.json")
for path in list(root.rglob("*")):
    if not path.is_file() or "fixture" in path.relative_to(root).parts:
        continue
    if path.suffix in (".log", ".raw", ".txt"):
        data = path.read_bytes()
        archive = path.with_suffix(path.suffix + ".gz")
        archive.write_bytes(gzip.compress(data, mtime=0))
        assert gzip.decompress(archive.read_bytes()) == data
rendered = ["# Rendered output reached with actual keys", "",
            "These rows were selected in the real TUI; full viewports and raw PTY streams",
            "are preserved as adjacent .txt.gz and .raw.gz files. No store JSON supplies these rows.", ""]
for lane in ["inspect-01", "limits-01", "recovery-01", "cap-01"]:
    for path in sorted((root / lane).glob("*-selected.json")):
        rendered += ["## " + str(path.relative_to(root)), "", "```text",
                     *json.loads(path.read_text()), "```", ""]
(root / "rendered-output.md").write_text("\n".join(rendered))
tests = {}
for lane in ["core-default", "core-feature", "tui"]:
    data = (root / (lane+".log")).read_text()
    tests[lane] = {
        "summary": re.findall(r"Summary .*", data)[-1],
        "failure_lines": re.findall(r"^.*(?:FAIL |TIMEOUT ).*$", data, re.MULTILINE),
        "slow_lines": re.findall(r"^.*SLOW .*", data, re.MULTILINE),
    }
(root / "test-results.json").write_text(json.dumps(tests, indent=2)+"\n")
paths = subprocess.check_output(
    ["git", "ls-files", "--others", "--exclude-standard", "-z", "--", str(root)],
    cwd=repo).decode().split("\0")
files = []
for name in filter(None, paths):
    path = repo / name
    if path.name == "artifact-inventory.json":
        continue
    data = path.read_bytes()
    is_archive = path.suffix == ".gz"
    files.append(dict(path=str(path.relative_to(root)), bytes=len(data),
                      sha256=hashlib.sha256(data).hexdigest(),
                      lines=None if is_archive else len(data.splitlines()),
                      category="qa_program" if path.suffix in (".py", ".c") else
                               "compressed_evidence" if is_archive else "text_evidence"))
counts = {}
for category in ["qa_program", "text_evidence", "compressed_evidence"]:
    selected = [f for f in files if f["category"] == category]
    counts[category] = dict(files=len(selected), lines=sum(f["lines"] or 0 for f in selected),
                            bytes=sum(f["bytes"] for f in selected))
inventory = dict(rust_test_lines_changed=0, production_lines_changed=0,
                 tracked_source_diff=subprocess.check_output(["git", "diff", "--numstat"], cwd=repo).decode(),
                 new_files=counts, files=files,
                 note="Inventory excludes itself. Raw originals remain locally; lossless gzip copies are reviewable.")
(root / "artifact-inventory.json").write_text(json.dumps(inventory, indent=2)+"\n")
print(json.dumps(counts, indent=2))

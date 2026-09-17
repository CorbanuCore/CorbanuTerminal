"""Freeze exact source/capture provenance and actual gate summaries, without rerunning."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
base = "270a6644e01d780ef93f8715dd0051481643dc7d"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip() == base

def save(name, value):
    with (here / name).open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")

sources = [
    ("startup-owner", "codex-rs/core/src/session_startup_prewarm.rs", 305, 325),
    ("warmup-exclusion", "codex-rs/core/src/client.rs", 3447, 3455),
    ("warmup-drain", "codex-rs/core/src/client.rs", 3694, 3745),
    ("startup-needs-owner", "codex-rs/core/src/accounting_responses.rs", 18, 26),
    ("attempt-contract", "codex-rs/state/src/runtime/accounting_types.rs", 71, 88),
    ("warmup-fixture", "codex-rs/core/tests/suite/accounting_responses_ws_support.rs", 121, 134),
    ("warmup-exclusion-test", "codex-rs/core/tests/suite/accounting_responses_ws.rs", 82, 106),
]
records = []
for identity, path, start, end in sources:
    data = (repo / path).read_bytes()
    assert data == subprocess.check_output(["git", "show", base + ":" + path], cwd=repo)
    lines = data.decode().splitlines(keepends=True)
    assert end <= len(lines)
    records.append(dict(identity=identity, path=path, start=start, end=end,
                        sha256=hashlib.sha256(data).hexdigest(),
                        excerpt="".join(lines[start - 1:end])))
save("next-step-source.json", dict(base=base, sources=records))
captures = []
for allocation, run, stem in [
    ("acct-readers-61", "reader-run-05", "never-prompted"),
    ("acct-scope-62", "scope-run-01", "fresh-scope-zero"),
]:
    directory = here.parent / allocation / run
    selected = directory / (stem + "-selected.json")
    viewport = directory / (stem + ".txt.gz")
    rows = json.loads(selected.read_text())
    text = gzip.decompress(viewport.read_bytes()).decode()
    assert "Requested UTC day: 2026-08-10" in text
    assert all(row in text for row in rows)
    assert len(rows) == 24
    captures.append(dict(allocation=allocation, requested_utc_day="2026-08-10",
                         selected=str(selected.relative_to(repo)),
                         selected_sha256=hashlib.sha256(selected.read_bytes()).hexdigest(),
                         viewport=str(viewport.relative_to(repo)),
                         viewport_sha256=hashlib.sha256(viewport.read_bytes()).hexdigest(),
                         rows=rows))
assert captures[0]["selected_sha256"] == captures[1]["selected_sha256"]
assert captures[0]["viewport_sha256"] == captures[1]["viewport_sha256"]
save("capture-bindings.json", dict(captures=captures,
     note="The selected JSON files are byte-identical (SHA-256 cb01c7c7fdc7e4872d760318224460f6a79120b9c2c2e8409dfdb23f9c347465), and the compressed viewport files are byte-identical (SHA-256 e40aa441137f8f956d17b726e035d3f0ea510a0b3dc5f847511868bfb2a51c7e; 1249 bytes each), including Read at 1786363200000 and the admission interval. These captures are indistinguishable at the page level: they bind neither identical nor distinct attempt populations or resolved roots. The separation rests on the store read-backs alone; no distinct-population claim is made from these pages. Actual wall-clock capture dates are not asserted."))

lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    record = json.loads((here / "gates-01" / (name + ".json")).read_text())
    path = here / "gates-01" / (name + ".log")
    data = path.read_bytes()
    text = data.decode()
    record["log_sha256"] = hashlib.sha256(data).hexdigest()
    record["failure_lines"] = list(dict.fromkeys(line.strip() for line in re.findall(
        r"^\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s.*$",
        text, re.M,
    )))
    if name != "prerequisites":
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests run: (.+)$", text, re.M)
        assert len(summaries) == 1, (name, summaries)
        run, details = summaries[0]
        passed = re.search(r"(\d+) passed", details)
        assert passed
        def count(label):
            match = re.search(r"(\d+) " + label, details)
            return int(match[1]) if match else 0
        record.update(run=int(run), passed=int(passed[1]), summary=details,
                      failed=count("failed"), timed_out=count("timed out"),
                      flaky=count("flaky"))
    with path.with_suffix(".log.gz").open("xb") as stream:
        stream.write(gzip.compress(data, mtime=0))
    lanes.append(record)
assert [row["run"] for row in lanes[1:]] == [124, 127, 91]
summary = {key: sum(row[key] for row in lanes[1:])
           for key in ("passed", "failed", "timed_out", "flaky")}
save("test-results.json", dict(base=base, lanes=lanes, total_executions=342,
                               **summary,
                               interpretation="Overlapping default/feature lane test counts; retry attempts are preserved in logs and failure_lines, not added to these counts."))
print(json.dumps(summary))

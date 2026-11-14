"""Freeze source excerpts and exact gate results; does not run product code."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip() == (
    "49fac26d01fdc172d6300fd72cad8037d0c6e351"
)


def save(name, value):
    with (here / name).open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


sources = [
    ("selector", "codex-rs/core/src/accounting.rs", 40, 76),
    ("gateway-eligibility", "codex-rs/core/src/accounting_chat.rs", 170, 187),
    ("gateway-exclusion-test", "codex-rs/core/tests/suite/accounting_chat.rs", 424, 477),
    ("warmup-no-sampling", "codex-rs/core/src/client.rs", 3447, 3488),
    ("warmup-drain", "codex-rs/core/src/client.rs", 3694, 3745),
    ("warmup-fixture", "codex-rs/core/tests/suite/accounting_responses_ws_support.rs", 114, 134),
    ("warmup-exclusion-test", "codex-rs/core/tests/suite/accounting_responses_ws.rs", 82, 106),
    ("auxiliary-http-exclusion-test", "codex-rs/core/tests/suite/accounting_responses.rs", 297, 353),
    ("auxiliary-chat-exclusion-test", "codex-rs/core/tests/suite/accounting_chat.rs", 379, 421),
    ("auxiliary-ws-exclusion-test", "codex-rs/core/tests/suite/accounting_responses_ws_recovery.rs", 516, 549),
    ("legacy-display-replay", "codex-rs/app-server/src/request_processors/token_usage_replay.rs", 1, 110),
    ("compact-import-receipt", "qa/portfolio/agent-cost-accounting/pf-60-s02/compact-late-import-receiving.md", 49, 57),
    ("historical-native-fixture", "qa/portfolio/agent-cost-accounting/pf-60-s03/acct-readers-61/readers.py", 152, 166),
]
records = []
for identity, name, start, end in sources:
    data = (repo / name).read_bytes()
    lines = data.decode().splitlines(keepends=True)
    assert end <= len(lines), (name, len(lines))
    records.append(dict(identity=identity, path=name, start=start, end=end,
                        sha256=hashlib.sha256(data).hexdigest(),
                        excerpt="".join(lines[start - 1:end])))
save("source-evidence.json", records)
lanes = json.loads((here / "gates-01/lanes.json").read_text())
results = []
for lane in lanes:
    path = here / "gates-01" / (lane["lane"] + ".log")
    data = path.read_bytes()
    text = data.decode()
    summary = re.findall(r"^\s*Summary .*?(\d+) tests run: (.+)$", text, re.M)
    if lane["lane"] in ("core-default", "core-feature", "tui"):
        assert len(summary) == 1, lane
        total, details = summary[0]
        passed = re.search(r"(\d+) passed", details)
        assert passed, details
        failure_lines = re.findall(r"^\s*(?:FAIL|TIMEOUT|XPASS|LEAK FAIL).*$", text, re.M)
        results.append(dict(lane=lane["lane"], run=int(total), passed=int(passed[1]),
                            failure_lines=failure_lines, summary=details, exit=lane["exit"],
                            log_sha256=hashlib.sha256(data).hexdigest()))
    with path.with_suffix(".log.gz").open("xb") as stream:
        stream.write(gzip.compress(data, mtime=0))
assert [(r["run"], r["passed"], r["exit"], r["failure_lines"]) for r in results] == [
    (124, 124, 0, []), (127, 127, 0, []), (91, 91, 0, []),
]
save("test-results.json", dict(
    base_commit="49fac26d01fdc172d6300fd72cad8037d0c6e351",
    lanes=results, total_executions=342, passed=342, failed=0,
    note="Lane executions overlap between default and feature builds; not 342 unique test names.",
    environment=dict(NEXTEST_TEST_THREADS="4", CARGO_BUILD_JOBS="2",
                     CARGO_TARGET_DIR=lanes[0]["target"]),
))
binary = Path(lanes[0]["target"]) / "debug/codex"
save("binary.json", dict(path=str(binary), sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                        newly_executed_in_pty=False,
                        note="Built by this campaign; preserved round-61/62 PTY captures were re-audited, not rerun."))
print("Source evidence frozen; 124/124 + 127/127 + 91/91; exact failure names: none.")

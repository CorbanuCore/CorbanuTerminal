"""Run one guarded lane; inspect its log before dispatching any successor."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

here = Path(__file__).resolve().parent
repo = here.parents[4]
test = "suite::accounting_responses_ws_recovery::accounting_responses_ws_native_auxiliary_scope_and_event_parity"
commands = {
    "prerequisites": ["cargo", "build", "--locked", "--offline", "-p", "codex-cli",
                      "--bin", "codex", "-p", "codex-rmcp-client",
                      "-p", "codex-code-mode-host", "--bins"],
    "alone-feature": ["just", "test", "-p", "codex-core", test,
                      "--features", "codex-core/developer-accounting"],
    "core-default": ["just", "test", "-p", "codex-core", "accounting"],
    "core-feature": ["just", "test", "-p", "codex-core", "accounting",
                     "--features", "codex-core/developer-accounting"],
    "tui": ["just", "test", "-p", "codex-tui", "usage"],
}
name = sys.argv[1]
out = here / "gates-02"
out.mkdir(exist_ok=True)
if name != "prerequisites":
    assert json.loads((out / "prerequisites.json").read_text())["exit"] == 0
settings = dict(
    CARGO_TARGET_DIR=str(here.parent / "acct-activation-33/feature/target"),
    CARGO_BUILD_JOBS="2", CARGO_INCREMENTAL="0", NEXTEST_TEST_THREADS="4",
    CARGO_PROFILE_DEV_DEBUG_ASSERTIONS="true",
    CARGO_PROFILE_TEST_DEBUG_ASSERTIONS="true",
)
start = time.time()
path = out / (name + ".log")
with path.open("x") as log:
    process = subprocess.run(commands[name], cwd=repo / "codex-rs",
                             env=dict(os.environ, **settings),
                             stdout=log, stderr=subprocess.STDOUT)
data = path.read_bytes()
text = data.decode()
record = dict(lane=name, command=commands[name], cwd=str(repo / "codex-rs"),
              base=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
              environment=settings, exit=process.returncode,
              elapsed_seconds=time.time() - start,
              log_sha256=hashlib.sha256(data).hexdigest(),
              failure_lines=list(dict.fromkeys(re.findall(
                  r"^\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s.*$",
                  text, re.M))),
              target_test_lines=[line.strip() for line in text.splitlines() if test in line])
if name != "prerequisites":
    summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
    record["summaries"] = summaries
    if len(summaries) == 1:
        run, details = summaries[0]
        record["run"] = int(run)
        for key, label in (("passed", "passed"), ("failed", "failed"),
                           ("timed_out", "timed out"), ("flaky", "flaky"),
                           ("skipped", "skipped"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, details)
            record[key] = int(match[1]) if match else 0
with path.with_suffix(".log.gz").open("xb") as stream:
    stream.write(gzip.compress(data, mtime=0))
with (out / (name + ".json")).open("x") as stream:
    json.dump(record, stream, indent=2)
    stream.write("\n")
print(json.dumps(record))
sys.exit(process.returncode)

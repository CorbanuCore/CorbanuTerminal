"""Collect completed block evidence without changing retained attempts."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
records = []
transcript = bytearray()
banner = "Test isolation: disposable profile; native keyring disabled (debug lane)"
for index, lane in enumerate(
    ("clone", "evidence", "prerequisites", "core-default", "core-feature", "tui"), 1
):
    record = json.loads((here / f"block-{index:02d}.json").read_text())
    raw = (here / f"block-{index:02d}.log").read_bytes()
    assert hashlib.sha256(raw).hexdigest() == record["log_sha256"]
    assert record["exit"] == 0, record
    record["lane"] = lane
    text = raw.decode()
    if index >= 4:
        assert banner in text, f"{lane}: missing isolation banner"
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
        assert len(summaries) == 1, summaries
        count, detail = summaries[0]
        record["run"] = int(count)
        for key, label in (("passed", "passed"), ("failed", "failed"),
                           ("timed_out", "timed out"), ("skipped", "skipped"),
                           ("flaky", "flaky"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + label, detail)
            record[key] = int(match[1]) if match else 0
        record["failure_names"] = re.findall(
            r"^\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s.*$", text, re.M
        )
        assert record["run"] > 0 and record["run"] == record["passed"]
        assert not record["failure_names"]
        assert record["failed"] == record["timed_out"] == 0
        record["isolation_banner"] = banner
    with (here / f"block-{index:02d}.log.gz").open("xb") as output:
        output.write(gzip.compress(raw, mtime=0))
    transcript.extend(
        f"===== BLOCK {index}: {lane} =====\nCWD: {record['cwd']}\n"
        "SHELL: bash --noprofile --norc -x -c <exact block below>\n"
        "----- VERBATIM INPUT -----\n".encode()
    )
    transcript.extend(record["command"][-1].encode())
    transcript.extend(b"----- VERBATIM MERGED STDOUT/STDERR -----\n")
    transcript.extend(raw)
    transcript.extend(f"----- EXIT: {record['exit']} -----\n\n".encode())
    records.append(record)
with (here / "transcript.txt").open("xb") as output:
    output.write(transcript)
audit = here / "target/audit-copy"
runs = list((audit / "qa/portfolio/agent-cost-accounting/pf-60-s03/acct-reference-88/target").glob("current-tip-*"))
assert len(runs) == 1, runs
replay = here / "replay-streams"
replay.mkdir()
for name in ("acceptance.stdout.txt", "acceptance.stderr.txt", "controls.stdout.txt",
             "controls.stderr.txt", "receipt.json"):
    shutil.copyfile(runs[0] / name, replay / name)
reference = "refs/heads/integrate/management-workstreams-20260911"
def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, text=True)
status = git("status", "--porcelain", "--untracked-files=all")
scope = "qa/portfolio/agent-cost-accounting/pf-60-s03/"
assert all(line[3:].startswith(scope) for line in status.splitlines()), status
assert git("status", "--porcelain", "--untracked-files=all", cwd=audit) == ""
note = here.parent / "acct-derive-95/OPERATOR.md"
assert note.read_bytes() == (here / "OPERATOR.executed.md").read_bytes()
old_note = git("show", "HEAD:" + str(note.relative_to(repo)))
for start, stop in (("This checks", "**Get a clean copy.**"),
                    ("**The three unavailable artifacts**", "**Fresh Rust gates")):
    assert old_note.split(start, 1)[1].split(stop, 1)[0] == note.read_text().split(start, 1)[1].split(stop, 1)[0]
result = {
    "base_commit": git("rev-parse", "HEAD").strip(),
    "candidate_commit": git("rev-parse", "HEAD", cwd=audit).strip(),
    "note_sha256": hashlib.sha256(note.read_bytes()).hexdigest(),
    "transcript_sha256": hashlib.sha256(transcript).hexdigest(),
    "source_integration_ref_after": git("rev-parse", reference).strip(),
    "clone_integration_ref": git("rev-parse", reference, cwd=audit).strip(),
    "source_status": status,
    "clone_status_after": "",
    "all_changes_within_scope": True,
    "honest_limits_unchanged": True,
    "blocks": records,
}
with (here / "results.json").open("x") as output:
    json.dump(result, output, indent=2)
    output.write("\n")
with (here / "changed-lines.patch").open("x") as output:
    output.write(git("diff", "--unified=0", "--", str(note.relative_to(repo))))
print(json.dumps({
    "lanes": [{key: row[key] for key in ("lane", "run", "passed", "skipped", "failed",
              "timed_out", "failure_names")} for row in records[3:]],
    "all_changes_within_scope": True,
    "transcript_sha256": result["transcript_sha256"],
}, indent=2))

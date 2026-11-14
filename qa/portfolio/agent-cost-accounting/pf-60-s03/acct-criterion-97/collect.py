"""Retain all attempts and derive final lane counts from raw nextest summaries."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

here = Path(__file__).resolve().parent
repo = here.parents[4]
scope = here.parent.relative_to(repo)
audit = here / "target/final-audit-copy"
notes = {}
for name in ("OPERATOR.initial.md", "OPERATOR.intermediate.md", "OPERATOR.executed.md"):
    raw = (here / name).read_bytes()
    notes[hashlib.sha256(raw).hexdigest()] = (name, re.findall(
        r"^```sh\n(.*?)^```$", raw.decode(), re.M | re.S))
final_hash = hashlib.sha256((here / "OPERATOR.executed.md").read_bytes()).hexdigest()
assert (here.parent / "acct-derive-95/OPERATOR.md").read_bytes() == (here / "OPERATOR.executed.md").read_bytes()
labels = ["00-source-clone", "01-tracked-stop", "02-omission-stop",
          "03-tracked-stop", "04-omission-stop", "block-01", "block-02",
          *[f"wrong-root-{i:02d}" for i in range(2, 7)],
          "existing-destination-stop", "second-destination", "block-03",
          "final-tracked-stop", "final-omission-stop", "final-staged-cancellation-stop",
          *[f"final-block-{i:02d}" for i in range(1, 7)],
          *[f"final-wrong-root-{i:02d}" for i in range(2, 7)],
          "final-existing-destination-stop", "final-second-destination",
          "final-default-destination"]
records = []
transcript = bytearray()
lanes = []
banner = "Test isolation: disposable profile; native keyring disabled (debug lane)"
for label in labels:
    record = json.loads((here / (label + ".json")).read_text())
    raw = (here / (label + ".log")).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == record["log_sha256"]
    script = record["command"][-1]
    assert hashlib.sha256(script.encode()).hexdigest() == record["script_sha256"]
    note_name, blocks = notes[record["note_sha256"]]
    if label != "00-source-clone":
        assert script in blocks
    if label.startswith("final-"):
        assert record["note_sha256"] == final_hash
    record["frozen_note"] = note_name
    if label == "block-03":
        intermediate = (here / "OPERATOR.intermediate.md").read_bytes()
        assert script in re.findall(r"^```sh\n(.*?)^```$", intermediate.decode(), re.M | re.S)
        record["launch_note"] = "OPERATOR.intermediate.md"
        record["launch_note_sha256"] = hashlib.sha256(intermediate).hexdigest()
        record["receipt_note_hash_sampled_at_completion"] = True
        record["script_identical_in_launch_and_final_notes"] = True
    record["expected_exit_matched"] = record["exit"] == record["expected_exit"]
    if label in ("final-block-04", "final-block-05", "final-block-06"):
        text = raw.decode()
        row = dict(label=label, exit=record["exit"], isolation_banner=banner in text)
        summaries = re.findall(r"^\s*Summary .*?(\d+) tests? run: (.+)$", text, re.M)
        assert len(summaries) == 1, (label, summaries)
        count, detail = summaries[0]
        row["run"] = int(count)
        for key, word in (("passed", "passed"), ("failed", "failed"),
                          ("timed_out", "timed out"), ("skipped", "skipped"),
                          ("flaky", "flaky"), ("leaky", "leaky")):
            match = re.search(r"(\d+) " + word, detail)
            row[key] = int(match[1]) if match else 0
        row["failure_names"] = list(dict.fromkeys(re.findall(
            r"^\s*(?:TRY\s+\d+\s+)?(?:FAIL|TIMEOUT|TMT|XPASS|LEAK FAIL)\s.*$", text, re.M)))
        assert row["run"] > 0 and row["isolation_banner"]
        lanes.append(row)
    with (here / (label + ".log.gz")).open("xb") as output:
        output.write(gzip.compress(raw, mtime=0))
    transcript.extend(("\n===== " + label + " =====\n"
        + "NOTE MATCHED BY RECEIPT HASH: " + note_name
        + "\nLAUNCH NOTE: " + record.get("launch_note", note_name)
        + "\nCWD: " + record["cwd"]
        + "\nEXPLICIT INPUTS: " + json.dumps(record["explicit_inputs"])
        + "\nSHELL: bash --noprofile --norc -x -c <verbatim input>\n"
        + "----- VERBATIM INPUT -----\n" + script
        + "----- VERBATIM MERGED STDOUT/STDERR -----\n").encode())
    transcript.extend(raw)
    transcript.extend(f"----- EXIT: {record['exit']} -----\n".encode())
    records.append(record)
with (here / "transcript.txt").open("xb") as output:
    output.write(transcript)
runs = list((audit / scope / "acct-reference-88/target").glob("current-tip-*"))
assert len(runs) == 1
replay = here / "replay-streams"
replay.mkdir()
for name in ("acceptance.stdout.txt", "acceptance.stderr.txt", "controls.stdout.txt",
             "controls.stderr.txt", "receipt.json"):
    shutil.copyfile(runs[0] / name, replay / name)
def git(*args, cwd=repo):
    return subprocess.check_output(["git", *args], cwd=cwd, text=True)
status = git("status", "--porcelain", "--untracked-files=all")
assert all(line[3:].startswith(str(scope) + "/") for line in status.splitlines())
audit_status = git("status", "--porcelain", "--untracked-files=all", cwd=audit)
assert audit_status == ""
result = dict(
    base_commit=git("rev-parse", "HEAD").strip(),
    candidate_commit=git("rev-parse", "HEAD", cwd=audit).strip(),
    note_sha256=final_hash, transcript_sha256=hashlib.sha256(transcript).hexdigest(),
    lanes=lanes, attempts=records, source_status=status, audit_status=audit_status,
    all_changes_within_scope=True,
    source_integration_ref=git("rev-parse", "refs/heads/integrate/management-workstreams-20260911").strip(),
    original_attempts_preserved=True)
with (here / "results.json").open("x") as output:
    json.dump(result, output, indent=2)
    output.write("\n")
with (here / "changed-lines.patch").open("x") as output:
    output.write(git("diff", "--unified=0", "--", str(scope / "acct-derive-95/OPERATOR.md")))
print(json.dumps({key: result[key] for key in (
    "note_sha256", "transcript_sha256", "lanes", "all_changes_within_scope")}, indent=2))

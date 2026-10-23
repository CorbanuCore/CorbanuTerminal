"""Exercise the operator note's destination guard with real Git ignore rules."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

here = Path(__file__).resolve().parent
note = here.parent / "acct-derive-95/OPERATOR.md"
old_commit = "967425cc72ea28b5f59ecc38a56419b7db6885e8"
new_commit = "a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a"
repo = here.parents[4]
note_path = str(note.relative_to(repo))
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", type=Path, help="New receipt path; existing files are refused.")
args = parser.parse_args()
# Match the current-note checker: do not inherit Git/profile configuration.
env = dict(PATH=os.environ["PATH"], GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1",
           GIT_TERMINAL_PROMPT="0", PYTHONDONTWRITEBYTECODE="1")
raw = subprocess.check_output(["git", "show", new_commit + ":" + note_path], cwd=repo, env=env)
block = re.findall(r"^```sh\n(.*?)^```$", raw.decode(), re.M | re.S)[0]
guard = block[block.index("audit=${CORBANU_AUDIT_DIR"):block.index("git clone --no-local")]
old = subprocess.check_output(
    ["git", "show", old_commit + ":" + note_path], cwd=repo, env=env, text=True
)
old_block = re.findall(r"^```sh\n(.*?)^```$", old, re.M | re.S)[0]
old_guard = old_block[old_block.index("audit=${CORBANU_AUDIT_DIR"):old_block.index("git clone --no-local")]
scratch = here / "target"
scratch.mkdir(exist_ok=True)
root = Path(tempfile.mkdtemp(prefix="destination-", dir=scratch))
source = root / "source"
source.mkdir()
subprocess.run(["git", "init", "-q", str(source)], env=env, check=True)
(source / ".gitignore").write_text("ignored/\n")
(source / "ignored").mkdir()
alias = root / "outside-alias"
alias.symlink_to(source, target_is_directory=True)
cases = [
    ("direct-unignored", source / "unignored", 1),
    ("alias-unignored-regression", alias / "unignored", 1),
    ("alias-ignored", alias / "ignored/new-audit", 0),
    ("dotdot-unignored", root / "../" / root.name / "source/unignored", 1),
    ("external", root / "external-audit", 0),
    ("direct-ignored", source / "ignored/new-audit", 0),
    ("existing-destination", source / "ignored", 1),
]
records = []
for name, destination, expected in cases:
    script = 'set -eu\nq=unused\n' + guard + "printf 'DESTINATION_ALLOWED\\n'\n"
    result = subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", script],
        cwd=source, env=dict(env, PWD=str(source), CORBANU_AUDIT_DIR=str(destination)),
        text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    row = dict(case=name, destination=str(destination), resolved=str(destination.resolve()),
               expected_exit=expected, exit=result.returncode, stdout=result.stdout,
               stderr=result.stderr, command=script)
    records.append(row)
    if name == "alias-unignored-regression":
        previous = subprocess.run(
            ["bash", "--noprofile", "--norc", "-c", "set -eu\nq=unused\n" + old_guard],
            cwd=source, env=dict(env, PWD=str(source), CORBANU_AUDIT_DIR=str(destination)),
            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )
        row["old_guard_exit"] = previous.returncode
        row["old_guard_stdout"] = previous.stdout
        row["old_guard_stderr"] = previous.stderr
        observed_old, observed_new = previous.returncode, result.returncode
        print(f"Observed alias-unignored-regression: old={observed_old}/new={observed_new}", flush=True)
        if (observed_old, observed_new) != (0, 1):
            raise RuntimeError(f"pinned regression exits differ from expected (0, 1): {row}")
    if result.returncode != expected:
        raise RuntimeError(f"destination exit differs from expected: {row}")
    if ("DESTINATION_ALLOWED" in result.stdout) != (expected == 0):
        raise RuntimeError(f"destination checkpoint differs from expected: {row}")
alias_records = [row for row in records if row["case"] == "alias-unignored-regression"]
if len(alias_records) != 1 or "old_guard_exit" not in alias_records[0]:
    raise RuntimeError("missing unique pinned alias-unignored-regression observation")
observed_old, observed_new = alias_records[0]["old_guard_exit"], alias_records[0]["exit"]
record = dict(old_commit=old_commit, new_commit=new_commit,
              old_note_sha256=hashlib.sha256(old.encode()).hexdigest(),
              old_guard_sha256=hashlib.sha256(old_guard.encode()).hexdigest(),
              new_guard_sha256=hashlib.sha256(guard.encode()).hexdigest(),
              note_sha256=hashlib.sha256(raw).hexdigest(),
              scope="Exact extracted destination segment, through the pre-clone guard; no clone/replay claim.",
              environment=env, fixture=str(root), passed=len(records), cases=records)
output_path = args.output if args.output else root / "destination-results.json"
with output_path.open("x") as output:
    json.dump(record, output, indent=2)
    output.write("\n")
print(json.dumps({"passed": len(records), "old_alias_exit": observed_old, "new_alias_exit": observed_new,
                  "receipt": str(output_path)}))

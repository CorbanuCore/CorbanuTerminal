"""Check note divergence, frozen receipt bindings, and current destination segment."""
import gzip
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
scope = here.parent
repo = here.parents[4]
def digest(raw):
    return hashlib.sha256(raw).hexdigest()
def blocks(raw):
    return re.findall(r"^```sh\n(.*?)^```$", raw.decode(), re.M | re.S)
note = (scope / "acct-derive-95/OPERATOR.md").read_bytes()
frozen = (scope / "acct-criterion-97/OPERATOR.executed.md").read_bytes()
live_blocks, frozen_blocks = blocks(note), blocks(frozen)
assert len(live_blocks) == len(frozen_blocks) == 6
assert digest(frozen) == "d46df323a5d5538c6b4e8bddec84aa08266346d5cc827bc7ac5581d204457864"
assert digest((scope / "acct-criterion-97/transcript.txt").read_bytes()) == "b44f78b644072cbbf49c6cf9e2d954baa097297e119b6906ae0ab5b0f271164f"
comparisons = []
for number, (live, old) in enumerate(zip(live_blocks, frozen_blocks), 1):
    receipt = json.loads((scope / ("acct-criterion-97/final-block-%02d.json" % number)).read_text())
    raw_log = gzip.decompress((scope / ("acct-criterion-97/final-block-%02d.log.gz" % number)).read_bytes())
    assert receipt["command"][-1] == old
    assert receipt["script_sha256"] == digest(old.encode())
    assert receipt["note_sha256"] == digest(frozen)
    assert receipt["log_sha256"] == digest(raw_log) and receipt["exit"] == 0
    subprocess.run(["bash", "--noprofile", "--norc", "-n"], input=live, text=True, check=True)
    comparisons.append(dict(block=number, equal=live == old,
                            frozen_sha256=digest(old.encode()), live_sha256=digest(live.encode()),
                            frozen_receipt_and_log_hashes_match=True))
assert [row["block"] for row in comparisons if not row["equal"]] == [1]
guard = live_blocks[0].split("audit=${CORBANU_AUDIT_DIR", 1)[1].split("git clone --no-local", 1)[0]
guard = "audit=${CORBANU_AUDIT_DIR" + guard
scratch = here / "target"
scratch.mkdir(exist_ok=True)
root = Path(tempfile.mkdtemp(prefix="guard-", dir=scratch))
source = root / "source"
source.mkdir()
env = dict(PATH=os.environ["PATH"], GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1",
           GIT_TERMINAL_PROMPT="0", PYTHONDONTWRITEBYTECODE="1")
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
script = "set -eu\nq=unused\n" + guard + "printf 'DESTINATION_ALLOWED\\n'\n"
for name, destination, expected in cases:
    result = subprocess.run(["bash", "--noprofile", "--norc", "-c", script], cwd=source,
                            env=dict(env, PWD=str(source), CORBANU_AUDIT_DIR=str(destination)),
                            text=True, capture_output=True)
    assert result.returncode == expected
    assert ("DESTINATION_ALLOWED" in result.stdout) == (expected == 0)
    assert not result.stderr
    records.append(dict(case=name, expected_exit=expected, exit=result.returncode,
                        destination=str(destination), stdout=result.stdout, stderr=result.stderr))
# Execute the exact Python heredoc under the installed interpreter with only
# sys.version_info overridden. This is a branch simulation, not an old-runtime run.
python_guard = guard.split("<<'PY'\n", 1)[1].split("\nPY\n", 1)[0]
versions = []
for version, expected in [((3, 8, 0), 1), ((3, 9, 0), 0)]:
    launcher = "import sys; sys.version_info = " + repr(version) + "; exec(compile(sys.stdin.read(), '<guard>', 'exec'))"
    command = [sys.executable, "-B", "-c", launcher, str(source), str(root / "new-external")]
    result = subprocess.run(command, input=python_guard, env=env, cwd=source, text=True, capture_output=True)
    assert result.returncode == expected and not result.stderr
    assert result.stdout == ("STOP: destination guard requires Python 3.9 or newer.\n" if expected else "")
    versions.append(dict(simulated_version=list(version), host_version=sys.version,
                         command=command, exit=result.returncode, stdout=result.stdout, stderr=result.stderr))
record = dict(note_sha256=digest(note), frozen_sha256=digest(frozen),
              scope="Byte/receipt checks plus current destination segment only; no clone or six-block execution.",
              blocks=comparisons, guard_script=script, destination_cases=records,
              version_test_scope="Simulated sys.version_info on installed Python; not actual Python 3.8/3.9 qualification.",
              python_guard=python_guard, version_cases=versions)
with (here / "note-check-final.json").open("x") as output:
    json.dump(record, output, indent=2)
    output.write("\n")
print(json.dumps(dict(changed_blocks=[1], identical_blocks=[2, 3, 4, 5, 6],
                      destination_cases_passed=len(records), simulated_version_cases_passed=len(versions),
                      note_sha256=digest(note), frozen_sha256=digest(frozen))))

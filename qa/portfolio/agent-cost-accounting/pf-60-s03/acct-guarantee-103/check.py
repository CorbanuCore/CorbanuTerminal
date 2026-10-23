"""Exercise observed witness exits, ambient-Git isolation and live refusal branches."""
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
env = dict(PATH=os.environ["PATH"], GIT_CONFIG_GLOBAL="/dev/null",
           GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0",
           PYTHONDONTWRITEBYTECODE="1")
def require(condition, message):
    if not condition:
        raise RuntimeError(message)
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
scratch = here / "target"
scratch.mkdir(exist_ok=True)
root = Path(tempfile.mkdtemp(prefix="checks-", dir=scratch))
witness = scope / "acct-final-100/test_destination.py"
runs = []
def run(name, command, run_env, expected, input_text=None, cwd=repo):
    result = subprocess.run(command, cwd=cwd, env=run_env, input=input_text,
                            text=True, capture_output=True)
    row = dict(name=name, command=command, exit=result.returncode,
               stdout=result.stdout, stderr=result.stderr, expected_exit=expected)
    (here / (name + ".stdout.txt")).write_text(result.stdout)
    (here / (name + ".stderr.txt")).write_text(result.stderr)
    runs.append(row)
    print(json.dumps(row), flush=True)
    require(result.returncode == expected, row)
    return result
clean = run("witness-clean", [sys.executable, "-B", str(witness),
            "--output", str(here / "witness-clean.json")], env, 0)
require("Observed alias-unignored-regression: old=0/new=1" in clean.stdout, "missing observations")
poison = dict(env, GIT_DIR=str(root / "absent-git-dir"), GIT_WORK_TREE=str(root / "wrong-tree"),
              GIT_INDEX_FILE=str(root / "absent-index"), GIT_CONFIG_COUNT="1",
              GIT_CONFIG_KEY_0="core.excludesFile", GIT_CONFIG_VALUE_0=str(root / "ignore-all"),
              GIT_CONFIG_GLOBAL=str(root / "bad-config"), BASH_ENV=str(root / "bad-shell"))
(root / "ignore-all").write_text("*\n")
(root / "bad-config").write_text("invalid configuration [\n")
(root / "bad-shell").write_text("exit 97\n")
polluted = run("witness-ambient-git", [sys.executable, "-B", str(witness),
               "--output", str(here / "witness-ambient-git.json")], poison, 0)
require("Observed alias-unignored-regression: old=0/new=1" in polluted.stdout, "ambient run changed")
# A mismatching old observation must fail, also when Python assertions are disabled.
raw = witness.read_text()
mutated = raw.replace('old_commit = "967425cc72ea28b5f59ecc38a56419b7db6885e8"',
                     'old_commit = "a8dfff98892e60aaa0d7f05321fd7e57b79cdc2a"')
require(mutated != raw, "mutation not applied")
launcher = "import sys; p=sys.argv.pop(1); exec(compile(sys.stdin.read(), p, 'exec'), {'__file__': p, '__name__': '__main__'})"
for flags, name in [([], "witness-mismatch"), (["-O"], "witness-mismatch-optimized")]:
    result = run(name, [sys.executable, "-B", *flags, "-c", launcher, str(witness),
                       "--output", str(root / (name + ".json"))], env, 1, mutated)
    require("old=1/new=1" in result.stdout and "pinned regression exits differ" in result.stderr,
            "wrong mismatch failure")
    require(not (root / (name + ".json")).exists(), "failed witness wrote passing receipt")
source = root / "source"
source.mkdir()
subprocess.run(["git", "init", "-q", str(source)], env=env, check=True)
(source / ".gitignore").write_text("ignored/\n")
(source / "ignored").mkdir()
alias = root / "outside-alias"
alias.symlink_to(source, target_is_directory=True)
dangling = root / "dangling"
dangling.symlink_to(root / "missing")
note = (scope / "acct-derive-95/OPERATOR.md").read_bytes()
frozen = (scope / "acct-criterion-97/OPERATOR.executed.md").read_bytes()
def blocks(raw):
    return re.findall(r"^```sh\n(.*?)^```$", raw.decode(), re.M | re.S)
live, old = blocks(note), blocks(frozen)
require(len(live) == len(old) == 6 and live[1:] == old[1:], "unexpected block divergence")
require(sha(frozen) == "d46df323a5d5538c6b4e8bddec84aa08266346d5cc827bc7ac5581d204457864", "frozen bytes changed")
for block in live:
    subprocess.run(["bash", "--noprofile", "--norc", "-n"], input=block,
                   text=True, env=env, check=True)
guard = live[0][live[0].index("audit=${CORBANU_AUDIT_DIR"):live[0].index("git clone --no-local")]
script = "set -eu\nq=unused\n" + guard + "printf 'DESTINATION_ALLOWED\\n'\n"
cases = [
    ("direct-unignored", source / "unignored", 1, "inside-source destination"),
    ("alias-unignored", alias / "unignored", 1, "inside-source destination"),
    ("dotdot-unignored", root / ".." / root.name / "source/unignored", 1, "inside-source destination"),
    ("existing-destination", source / "ignored", 1, "destination already exists"),
    ("dangling-symlink", dangling, 1, "destination is a symlink"),
    ("relative", Path("relative"), 1, "must be absolute"),
    ("direct-ignored", source / "ignored/new", 0, None),
    ("alias-ignored", alias / "ignored/new", 0, None),
    ("external", root / "new-external", 0, None),
]
for name, destination, expected, message in cases:
    result = run("guard-" + name, ["bash", "--noprofile", "--norc", "-c", script],
                 dict(env, PWD=str(source), CORBANU_AUDIT_DIR=str(destination)), expected, cwd=source)
    require(not result.stderr, name)
    require(("DESTINATION_ALLOWED" in result.stdout) == (expected == 0), name)
    if message:
        require("STOP:" in result.stdout and message in result.stdout, name)
python_guard = guard.split("<<'PY'\n", 1)[1].split("\nPY\n", 1)[0]
for version, expected in [((3, 8, 0), 1), ((3, 9, 0), 0)]:
    launcher = "import sys; sys.version_info = " + repr(version) + "; exec(compile(sys.stdin.read(), '<guard>', 'exec'))"
    result = run("guard-version-" + str(version[1]),
                 [sys.executable, "-B", "-c", launcher, str(source), str(root / "external")],
                 env, expected, python_guard, source)
    require(result.stdout == ("STOP: destination guard requires Python 3.9 or newer.\n" if expected else ""),
            "version stdout")
    require(not result.stderr, "version stderr")
record = dict(environment=env, synthetic_ambient_overrides=poison,
              witness_sha256=sha(witness.read_bytes()), note_sha256=sha(note),
              frozen_sha256=sha(frozen), identical_blocks=[2, 3, 4, 5, 6],
              scope="Destination segments only; version branches simulated on installed Python. No complete operator replay.",
              runs=runs)
with (here / "checks.json").open("x") as output:
    json.dump(record, output, indent=2)
    output.write("\n")
print("Checks passed; raw observations and negative controls retained.")

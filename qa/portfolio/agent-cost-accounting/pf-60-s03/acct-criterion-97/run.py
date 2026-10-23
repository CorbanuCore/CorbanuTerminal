"""Run frozen note blocks unchanged in fresh shells; preserve every attempt."""
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
scope = here.parent.relative_to(repo)
note = here.parent / "acct-derive-95/OPERATOR.md"
frozen = here / "OPERATOR.executed.md"
if not frozen.exists():
    frozen.write_bytes(note.read_bytes())
frozen_bytes = frozen.read_bytes()
assert frozen_bytes == note.read_bytes(), "Note changed; start a new attempt"
blocks = re.findall(r"^```sh\n(.*?)^```$", frozen.read_text(), re.M | re.S)
assert len(blocks) == 6
source = here / "target/source-copy"
audit = here / "target/final-audit-copy"
allowed = ("PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "SDKROOT",
           "MACOSX_DEPLOYMENT_TARGET", "DEVELOPER_DIR", "TMPDIR")
env = {key: os.environ[key] for key in allowed if key in os.environ}
env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
           GIT_TERMINAL_PROMPT="0", PYTHONDONTWRITEBYTECODE="1",
           CARGO_TERM_COLOR="never", TERM="dumb", LC_ALL="C")
assessment = (
    str(scope / "acct-criterion-97/deliberate-omission.txt")
    + " is a synthetic text output created solely to exercise this status branch; "
    "it is absent from the candidate inventories and replay/build input references. "
    + str(scope / "acct-criterion-97/target/")
    + " contains only our synthetic ignored scratch output. Neither changes any "
    "candidate claim or input. Both are deliberately omitted; only committed HEAD is replayed."
)

def run(label, script, cwd, expected=0, extra=None):
    command = ["bash", "--noprofile", "--norc", "-x", "-c", script]
    started = time.time()
    with (here / (label + ".log")).open("xb") as log:
        result = subprocess.run(command, cwd=cwd, env=dict(env, **(extra or {})),
                                stdout=log, stderr=subprocess.STDOUT)
    raw = (here / (label + ".log")).read_bytes()
    record = dict(label=label, command=command, cwd=str(cwd),
                  note_sha256=hashlib.sha256(frozen_bytes).hexdigest(),
                  script_sha256=hashlib.sha256(script.encode()).hexdigest(),
                  log_sha256=hashlib.sha256(raw).hexdigest(),
                  exit=result.returncode, expected_exit=expected,
                  elapsed_seconds=time.time() - started,
                  fresh_shell=True, environment_keys=sorted(dict(env, **(extra or {}))),
                  explicit_inputs=extra or {})
    (here / (label + ".json")).write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2), flush=True)
    assert result.returncode == expected, (label, result.returncode, expected)
    return record

mode = sys.argv[1]
if mode == "setup":
    here.joinpath("target").mkdir(exist_ok=True)
    assert not source.exists()
    base = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip()
    assert base == "2903d37c60dff3b8bec469fcf82347729b53f7fd"
    script = (
        'git clone --no-local --no-checkout "$SOURCE_REPO" "$SOURCE_COPY"\n'
        'git -C "$SOURCE_COPY" checkout --detach "$BASE"\n'
    )
    run("00-source-clone", "set -eu\n" + script, repo,
        extra=dict(SOURCE_REPO=str(repo), SOURCE_COPY=str(source), BASE=base))
    changed = source / scope / "acct-derive-95/OPERATOR.md"
    original = changed.read_bytes()
    changed.write_bytes(original + b"\nDeliberate tracked-change refusal fixture.\n")
    run("01-tracked-stop", blocks[0], source, expected=1)
    changed.write_bytes(original)
    omitted = source / scope / "acct-criterion-97/deliberate-omission.txt"
    omitted.parent.mkdir(exist_ok=True)
    omitted.write_text("Synthetic output; deliberately not a replay or build input.\n")
    scratch = omitted.parent / "target/scratch.txt"
    scratch.parent.mkdir(exist_ok=True)
    scratch.write_text("Synthetic ignored scratch output.\n")
    run("02-omission-stop", blocks[0], source, expected=1)
    # Do not copy the revised note or any evidence over committed candidate files.
elif mode == "status":
    changed = source / scope / "acct-derive-95/OPERATOR.md"
    original = changed.read_bytes()
    changed.write_bytes(original + b"\nDeliberate tracked-change refusal fixture.\n")
    run("final-tracked-stop", blocks[0], source, expected=1)
    changed.write_bytes(original)
    run("final-omission-stop", blocks[0], source, expected=1)
    changed.write_bytes(original + b"\nDeliberate staged fixture.\n")
    subprocess.run(["git", "add", str(changed.relative_to(source))],
                   cwd=source, env=env, check=True)
    changed.write_bytes(original)
    run("final-staged-cancellation-stop", blocks[0], source, expected=1,
        extra=dict(REPLAY_OMISSIONS_NOTE=assessment))
    subprocess.run(["git", "restore", "--staged", str(changed.relative_to(source))],
                   cwd=source, env=env, check=True)
elif mode == "1":
    run("final-block-01", blocks[0], source,
        extra=dict(CORBANU_AUDIT_DIR=str(audit), REPLAY_OMISSIONS_NOTE=assessment))
elif mode in ("2", "3", "4", "5", "6"):
    index = int(mode)
    prior = json.loads((here / f"final-block-{index - 1:02d}.json").read_text())
    assert prior["exit"] == 0, "Inspect failed predecessor before dispatch"
    run(f"final-block-{index:02d}", blocks[index - 1], audit)
elif mode == "default":
    run("final-default-destination", blocks[0], source,
        extra=dict(REPLAY_OMISSIONS_NOTE=assessment))
elif mode == "guards":
    for index in range(1, 6):
        run(f"final-wrong-root-{index + 1:02d}", blocks[index], source, expected=1)
    run("final-existing-destination-stop", blocks[0], source, expected=1,
        extra=dict(CORBANU_AUDIT_DIR=str(audit), REPLAY_OMISSIONS_NOTE=assessment))
    second = here / "target/final-audit-second-copy"
    run("final-second-destination", blocks[0], source,
        extra=dict(CORBANU_AUDIT_DIR=str(second), REPLAY_OMISSIONS_NOTE=assessment))
else:
    raise SystemExit("Unknown mode")

"""Execute one frozen OPERATOR.md shell block verbatim; inspect before successor."""
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
note = here.parent / "acct-derive-95/OPERATOR.md"
frozen = here / "OPERATOR.executed.md"
if not frozen.exists():
    frozen.write_bytes(note.read_bytes())
if frozen.read_bytes() != note.read_bytes():
    raise SystemExit("Note changed after freeze; preserve this attempt and start a new one")
blocks = re.findall(r"^```sh\n(.*?)^```$", frozen.read_text(), re.M | re.S)
index = int(sys.argv[1])
assert len(blocks) == 6 and 1 <= index <= len(blocks)
out = here / f"block-{index:02d}.log"
assert not out.exists(), "Never overwrite an attempt"
if index > 1:
    assert json.loads((here / f"block-{index - 1:02d}.json").read_text())["exit"] == 0
audit = here / "target/audit-copy"
cwd = repo if index == 1 else audit
# Preserve toolchain/dependency cache locations, not inference/profile aliases.
allowed = ("PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "SDKROOT",
           "MACOSX_DEPLOYMENT_TARGET", "DEVELOPER_DIR", "TMPDIR")
env = {key: os.environ[key] for key in allowed if key in os.environ}
env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
           GIT_TERMINAL_PROMPT="0", PYTHONDONTWRITEBYTECODE="1",
           CARGO_TERM_COLOR="never", TERM="dumb", LC_ALL="C")
command = ["bash", "--noprofile", "--norc", "-x", "-c", blocks[index - 1]]
started = time.time()
with out.open("xb") as log:
    process = subprocess.run(command, cwd=cwd, env=env,
                             stdout=log, stderr=subprocess.STDOUT)
raw = out.read_bytes()
record = {
    "block": index, "cwd": str(cwd), "command": command,
    "note_sha256": hashlib.sha256(frozen.read_bytes()).hexdigest(),
    "script_sha256": hashlib.sha256(blocks[index - 1].encode()).hexdigest(),
    "exit": process.returncode, "elapsed_seconds": time.time() - started,
    "log_sha256": hashlib.sha256(raw).hexdigest(),
    "environment_keys": sorted(env),
    "fresh_shell": True, "shell_variables_from_previous_block": False,
}
(here / f"block-{index:02d}.json").write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))
sys.exit(process.returncode)

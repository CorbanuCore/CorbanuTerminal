"""Run one guarded lane at a time; inspect its result before dispatching another."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

here = Path(__file__).resolve().parent
repo = here.parents[4]
target = here.parent / "acct-activation-33/feature/target"
commands = {
    "prerequisites": ["cargo", "build", "--locked", "--offline", "-p", "codex-cli",
                      "--bin", "codex", "-p", "codex-rmcp-client", "--bins"],
    "core-default": ["just", "test", "-p", "codex-core", "accounting"],
    "core-feature": ["just", "test", "-p", "codex-core", "accounting",
                     "--features", "codex-core/developer-accounting"],
    "tui": ["just", "test", "-p", "codex-tui", "usage"],
}
name = sys.argv[1]
out = here / "gates-01"
out.mkdir(exist_ok=True)
if name != "prerequisites":
    assert json.loads((out / "prerequisites.json").read_text())["exit"] == 0
settings = dict(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2",
                CARGO_INCREMENTAL="0", NEXTEST_TEST_THREADS="4",
                CARGO_PROFILE_DEV_DEBUG_ASSERTIONS="true",
                CARGO_PROFILE_TEST_DEBUG_ASSERTIONS="true")
env = dict(os.environ, **settings)
start = time.time()
with (out / (name + ".log")).open("x") as log:
    process = subprocess.run(commands[name], cwd=repo / "codex-rs", env=env,
                             stdout=log, stderr=subprocess.STDOUT)
record = dict(lane=name, command=commands[name], cwd=str(repo / "codex-rs"),
              environment=settings, exit=process.returncode,
              elapsed_seconds=time.time() - start)
with (out / (name + ".json")).open("x") as stream:
    json.dump(record, stream, indent=2)
    stream.write("\n")
print(json.dumps(record))
sys.exit(process.returncode)

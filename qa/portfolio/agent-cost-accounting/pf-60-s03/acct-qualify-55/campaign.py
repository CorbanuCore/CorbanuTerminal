"""Run the frozen gated lanes, preserving every exit and log."""
import json
import os
from pathlib import Path
import subprocess
import time

out = Path(__file__).resolve().parent
repo = out.parents[4]
target = out.parent / "acct-activation-33/feature/target"
env = os.environ.copy()
env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2",
           CARGO_INCREMENTAL="0", NEXTEST_TEST_THREADS="1",
           CARGO_PROFILE_DEV_DEBUG_ASSERTIONS="true",
           CARGO_PROFILE_TEST_DEBUG_ASSERTIONS="true")
lanes = [
    ("prerequisites", ["cargo", "build", "--locked", "--offline", "-p", "codex-cli", "--bin", "codex", "-p", "codex-rmcp-client", "--bins"]),
    ("core-default", ["just", "test", "-p", "codex-core", "accounting"]),
    ("core-feature", ["just", "test", "-p", "codex-core", "accounting", "--features", "codex-core/developer-accounting"]),
    ("tui", ["just", "test", "-p", "codex-tui", "usage"]),
    ("feature-binary", ["cargo", "build", "--locked", "--offline", "-p", "codex-cli", "--bin", "codex", "--features", "codex-core/developer-accounting"]),
]
results = []
for name, cmd in lanes:
    start = time.time()
    print(f"START {name}", flush=True)
    with (out / f"{name}.log").open("w") as log:
        proc = subprocess.run(cmd, cwd=repo / "codex-rs", env=env, stdout=log, stderr=subprocess.STDOUT)
    results.append(dict(lane=name, command=cmd, exit=proc.returncode,
                        elapsed=time.time()-start, target=str(target)))
    (out / "lanes.json").write_text(json.dumps(results, indent=2)+"\n")
    print(f"END {name}: {proc.returncode}", flush=True)
    if name == "prerequisites" and proc.returncode:
        break

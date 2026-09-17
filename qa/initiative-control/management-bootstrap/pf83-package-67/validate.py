"""Guarded requested lanes, with caches and logs confined to this allocation."""
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
ART = HERE / "artifacts"
TARGET = ART / "test-target"
RUST = Path("/Users/Neo/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin")


def main():
    os.umask(0o077)
    commands = [["/bin/cp", "-cR",
                 "/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31", str(TARGET)]]
    env = dict(PATH=str(RUST) + ":/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
               HOME=str(ART / "build-home"), TMPDIR=str(ART / "tmp"),
               CARGO_HOME=str(ART / "cargo-home"), CARGO_TARGET_DIR=str(TARGET),
               XDG_CACHE_HOME=str(ART / "cache"), NEXTEST_TEST_THREADS="4", INSTA_UPDATE="no")
    lane = sys.argv[1] if len(sys.argv) == 2 else "prerequisites"
    results = (json.loads((ART / "test-commands.json").read_text())["results"]
               if lane != "prerequisites" else [])
    def run(label, argv):
        with (ART / (label + ".log")).open("xb") as out:
            result = subprocess.run(argv, cwd=REPO / "codex-rs", env=env,
                                    stdout=out, stderr=subprocess.STDOUT)
        results.append(dict(lane=label, argv=argv, exit_code=result.returncode))
        (ART / "test-commands.json").write_text(json.dumps(dict(cwd=str(REPO / "codex-rs"),
            environment=env, results=results), indent=2) + "\n")
        print(label, result.returncode, flush=True)
        return result.returncode
    if lane != "prerequisites":
        filters = {"thread-settings": ("codex-app-server", "thread_settings"),
                   "permission-confirmation": ("codex-tui", "permission_confirmation")}
        assert lane in filters
        assert any(r["lane"] == "prerequisites" and r["exit_code"] == 0 for r in results)
        package, name = filters[lane]
        return run(lane, ["just", "test", "-p", package, name, "--locked", "--offline", "--retries", "0"])
    if run("test-cache-copy", commands[0]):
        return 1
    if run("prerequisites", ["cargo", "build", "--locked", "--offline", "-p", "codex-cli",
                             "-p", "codex-rmcp-client", "-p", "codex-code-mode-host", "--bins"]):
        return 1
    # Separate invocations allow prompt/contamination inspection between lanes.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

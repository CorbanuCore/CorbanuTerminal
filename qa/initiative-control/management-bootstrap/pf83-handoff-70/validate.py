"""Run one guarded gate stage; inspect its log before dispatching its successor."""
import json
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
PREVIOUS = HERE.parent / "pf83-package-67" / "artifacts"


def main():
    os.umask(0o077)
    stage = sys.argv[1]
    commands = {
        "prerequisites": ["cargo", "build", "--locked", "--offline", "-p", "codex-cli",
                          "-p", "codex-rmcp-client", "-p", "codex-code-mode-host", "--bins"],
        "thread-settings": ["just", "test", "-p", "codex-app-server", "thread_settings",
                            "--locked", "--offline", "--retries", "0"],
        "permission-confirmation": ["just", "test", "-p", "codex-tui", "permission_confirmation",
                                   "--locked", "--offline", "--retries", "0"],
    }
    env = json.loads((PREVIOUS / "test-commands.json").read_text())["environment"]
    assert env["CARGO_TARGET_DIR"] == str(PREVIOUS / "test-target")
    assert env["NEXTEST_TEST_THREADS"] == "4"
    assert set(env) == {"PATH", "HOME", "TMPDIR", "CARGO_HOME", "CARGO_TARGET_DIR",
                        "XDG_CACHE_HOME", "NEXTEST_TEST_THREADS", "INSTA_UPDATE"}
    if stage != "prerequisites":
        assert json.loads((HERE / "prerequisites-command.json").read_text())["exit_code"] == 0
    argv = commands[stage]
    with (HERE / (stage + ".log")).open("xb") as stream:
        result = subprocess.run(argv, cwd=REPO / "codex-rs", env=env,
                                stdout=stream, stderr=subprocess.STDOUT)
    record = dict(argv=argv, cwd=str(REPO / "codex-rs"), environment=env,
                  exit_code=result.returncode)
    with (HERE / (stage + "-command.json")).open("x") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    print(json.dumps(dict(stage=stage, exit_code=result.returncode)), flush=True)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())

"""Owner-run offline PTY regression, not code-blind acceptance.

Only synthetic profiles; never point this at an operator profile. Exact binary
input, private TMUX socket, clean environment and native-keyring guard. Logs and
failed attempts are retained in a new evidence directory. No inference is sent.
"""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import time
import tomllib
import uuid


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--startup", action="store_true")
    parser.add_argument("--startup-select", action="store_true")
    parser.add_argument("--remember-model-tier", action="store_true")
    args = parser.parse_args()
    args.evidence.mkdir(parents=True, exist_ok=False)
    root = args.evidence.resolve()
    home = root / "profile"
    project = root / "P"
    other = root / "Q"
    for directory in (home, project, other):
        directory.mkdir()
    binary = args.binary.resolve(strict=True)
    result = {"binary": str(binary), "sha256": hashlib.file_digest(binary.open("rb"), "sha256").hexdigest(),
              "passed": False, "startup": args.startup, "startup_select": args.startup_select,
              "checks": [], "independent_acceptance": False}
    config = ['model = "gpt-5.6-sol"', 'model_provider = "fixture"',
              'cli_auth_credentials_store = "file"', 'check_for_update_on_startup = false']
    if args.remember_model_tier:
        config[0] = 'model = "gpt-5.4"'
        config += ['service_tier = "fast"', '[features]', 'fast_mode = true']
    for provider in ("fixture", "fixture-claude", "fixture-fable"):
        config += [f"[model_providers.{provider}]", f'name = "{provider}"',
                   'base_url = "http://127.0.0.1:1/v1"', 'wire_api = "responses"',
                   'requires_openai_auth = false']
    for directory in (project, other):
        config += [f"[projects.{json.dumps(str(directory))}]", 'trust_level = "trusted"']
    (home / "config.toml").write_text("\n".join(config) + "\n")
    for index, (label, provider, cwd) in enumerate((
        ("ALPHA_OPENAI", "fixture", project),
        ("BRAVO_CLAUDE", "fixture-claude", project),
        ("CHARLIE_FABLE", "fixture-fable", project),
        ("DELTA_OTHER", "fixture", other),
    )):
        ident = str(uuid.uuid4())
        stamp = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(minutes=10-index)
        timestamp = stamp.isoformat(timespec="milliseconds").replace("+00:00", "Z")
        folder = home / "sessions" / stamp.strftime("%Y/%m/%d")
        folder.mkdir(parents=True, exist_ok=True)
        records = [
            ("session_meta", {"id": ident, "session_id": ident, "timestamp": timestamp,
                              "cwd": str(cwd), "originator": "corbanu_cli_rs", "cli_version": "0.1.42",
                              "source": "cli", "model_provider": provider, "base_instructions": {"text": "Synthetic QA only."}}),
            ("turn_context", {"cwd": str(cwd), "model": "fixture-fable-model" if args.remember_model_tier and provider != "fixture" else "gpt-5.6-sol", "model_provider": provider,
                              "approval_policy": "never", "sandbox_policy": {"type": "read-only"}, "summary": "auto"}),
            ("response_item", {"type": "message", "role": "user", "content": [{"type": "input_text", "text": label}]}),
            ("event_msg", {"type": "user_message", "message": label, "images": [], "local_images": []}),
            ("response_item", {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "SAVED_CONTEXT_" + label}]}),
            ("event_msg", {"type": "agent_message", "message": "SAVED_CONTEXT_" + label}),
        ]
        path = folder / f"rollout-{stamp.strftime('%Y-%m-%dT%H-%M-%S')}-{ident}.jsonl"
        path.write_text("".join(json.dumps({"timestamp": timestamp, "type": kind, "payload": payload}) + "\n" for kind, payload in records))
    # Short private socket path avoids macOS sockaddr_un's length limit.
    import tempfile
    socket_dir = Path(tempfile.mkdtemp(prefix="resume-pty-"))
    socket = socket_dir / "t"
    tmux_bin = shutil.which("tmux")
    env = {"HOME": str(home), "CORBANU_HOME": str(home), "CODEX_HOME": str(home),
           "PFTERMINAL_HOME": str(home), "CORBANU_TEST_NO_NATIVE_KEYRING": "1",
           "PATH": "/opt/homebrew/bin:/usr/bin:/bin", "TERM": "xterm-256color",
           "LANG": "en_US.UTF-8", "RUST_LOG": "trace", "TMPDIR": str(socket_dir)}
    actions = []

    def tmux(*words, check=True):
        return subprocess.run([tmux_bin, "-S", str(socket), *words], env=env,
                              text=True, capture_output=True, check=check, timeout=10).stdout

    def capture():
        return tmux("capture-pane", "-p", "-t", "probe")

    def keys(*words):
        actions.append({"at": time.time(), "keys": words})
        tmux("send-keys", "-t", "probe", *words)

    def text(value):
        keys("-l", "--", value)

    def command(value):
        text(value)
        time.sleep(.15)
        keys("Enter")

    def wait(marker, label, timeout=15):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            screen = capture()
            if marker in screen:
                (root / (label + ".txt")).write_text(screen)
                return screen
            time.sleep(.1)
        (root / (label + "-failed.txt")).write_text(capture())
        raise AssertionError(f"Missing {marker}: {label}")

    def passed(name):
        result["checks"].append(name)

    argv = [str(binary), "-c", "log_dir=" + json.dumps(str(root / "logs"))]
    if args.startup or args.startup_select:
        argv += ["resume"]
    # Secondary native guard: no network or access to the personal credential
    # directories. This does not purport to isolate the human/agent executor.
    if os.uname().sysname == "Darwin":
        policy = '(version 1)(allow default)(deny network*)(deny file-read* file-write* (subpath "/Users/Neo/Library/Keychains") (subpath "/Users/Neo/.codex") (subpath "/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home"))(deny mach-lookup (global-name "com.apple.securityd"))'
        argv = ["/usr/bin/sandbox-exec", "-p", policy, *argv]

    def check_remembered_and_restart(provider):
        time.sleep(.5)
        screen = capture()
        (root / "resumed-tier-check.txt").write_text(screen)
        assert "Configured service tier" not in screen, screen
        saved = tomllib.loads((home / "config.toml").read_text())
        assert (saved["model"], saved["model_provider"], saved["service_tier"]) == (
            "fixture-fable-model", provider, "default"), saved
        (root / "saved-defaults.json").write_text(json.dumps({k: saved.get(k) for k in (
            "model", "model_provider", "model_reasoning_effort", "service_tier")}, indent=2))
        command("/quit")
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            probe = subprocess.run([tmux_bin, "-S", str(socket), "has-session", "-t", "probe"],
                                   env=env, capture_output=True)
            if probe.returncode != 0:
                break
            time.sleep(.1)
        assert probe.returncode != 0, "normal quit did not finish"
        restart_argv = list(argv)
        if args.startup or args.startup_select:
            assert restart_argv[-1] == "resume"
            restart_argv.pop()
        tmux("new-session", "-d", "-s", "probe", "-x", "150", "-y", "45", "-c", str(project), "exec " + shlex.join(restart_argv))
        wait("Corbanu Terminal", "fresh-restart")
        time.sleep(1)
        command("/status")
        screen = wait(provider, "fresh-restart-provider")
        assert "fixture-fable-model" in screen, screen
        assert "Configured service tier" not in screen, screen
        passed("resume remembers model/provider/standard tier; normal quit and fresh launch restore them")
    try:
        tmux("new-session", "-d", "-s", "probe", "-x", "150", "-y", "45", "-c", str(project), "exec " + shlex.join(argv))
        if not (args.startup or args.startup_select):
            wait("Corbanu Terminal", "startup")
            time.sleep(1)
            command("/resume")
        screen = wait("CHARLIE_FABLE", "picker-cross-provider")
        assert "BRAVO_CLAUDE" in screen and "ALPHA_OPENAI" in screen, screen
        assert "DELTA_OTHER" not in screen, screen
        passed("same-directory sessions visible across three providers")
        text("NO_SUCH_SESSION_ZZZ")
        wait("No", "search-empty")
        for _ in "NO_SUCH_SESSION_ZZZ":
            keys("BSpace")
        wait("CHARLIE_FABLE", "search-cleared")
        text("BRAVO_CLAUDE")
        screen = wait("BRAVO_CLAUDE", "search-bravo")
        time.sleep(.5)
        screen = capture()
        assert "CHARLIE_FABLE" not in screen and "ALPHA_OPENAI" not in screen, screen
        passed("typing, backspace and search filter respond")
        if args.startup_select:
            keys("Enter")
            wait("SAVED_CONTEXT_BRAVO_CLAUDE", "startup-restored-history")
            text("UNSENT_STARTUP_DRAFT")
            wait("UNSENT_STARTUP_DRAFT", "startup-editable-composer")
            passed("startup selection restores alternate-provider history and editable chat")
            if args.remember_model_tier:
                for _ in "UNSENT_STARTUP_DRAFT":
                    keys("BSpace")
                check_remembered_and_restart("fixture-claude")
            result["passed"] = True
            return
        keys("Escape")
        wait("Type to search", "escape-clears-search")
        keys("Escape")
        wait("Corbanu Terminal", "cancelled-to-chat")
        passed("cancel returns to usable chat without selecting the filtered session")
        time.sleep(.3)
        command("/resume")
        wait("CHARLIE_FABLE", "reopened")
        text("CHARLIE_FABLE")
        time.sleep(.5)
        keys("Enter")
        wait("SAVED_CONTEXT_CHARLIE_FABLE", "restored-history")
        command("/status")
        wait("fixture-fable", "restored-provider")
        draft = "UNSENT_DRAFT_RESUME_OK"
        text(draft)
        wait(draft, "editable-composer")
        passed("reopen, select cross-provider session, restore history/provider and edit")
        for _ in draft:
            keys("BSpace")
        if args.remember_model_tier:
            check_remembered_and_restart("fixture-fable")
            result["passed"] = True
            return
        command("/resume")
        wait("CHARLIE_FABLE", "reopened-after-resume")
        keys("Right")  # Filter is the initially focused toolbar control.
        wait("DELTA_OTHER", "all-directories")
        text("DELTA_OTHER")
        time.sleep(.5)
        keys("Enter")
        wait("Choose working directory", "directory-choice")
        keys("Down")
        keys("Up")
        keys("Enter")
        wait("SAVED_CONTEXT_DELTA_OTHER", "other-directory-resumed")
        passed("all-directory selection and its directory prompt accept navigation and Enter")
        result["passed"] = True
    except BaseException as error:
        result["error"] = str(error)
        raise
    finally:
        (root / "actions.json").write_text(json.dumps(actions, indent=2) + "\n")
        (root / "result.json").write_text(json.dumps(result, indent=2) + "\n")
        tmux("kill-server", check=False)
    print(json.dumps(result))


if __name__ == "__main__":
    main()

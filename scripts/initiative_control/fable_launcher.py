#!/usr/bin/env python3
"""PF80 one-shot fresh Fable TUI sidecar. Import/help do not launch or authenticate."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import time
import uuid

MODEL, PROVIDER, EFFORT = "claude-fable-5-1-plan", "claude-plan", "high"
BRIEF_LIMIT, FINAL_LIMIT, RECORD_LIMIT = 65536, 131072, 16 * 1024 * 1024
SAFE_PATH = "/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:/usr/local/bin"
SECRET = re.compile(r"sk-(?:ant-|proj-)?[A-Za-z0-9_-]{12,}|Bearer\s+\S+", re.I)
INSTRUCTIONS = (
    "You are a bounded management decision maker. Use only the supplied JSON briefing. "
    "Do not use tools, read other files, edit code, launch workers, or take external actions. "
    "Return exactly one JSON object, without prose: state_revision (integer), actions (list). "
    "Every action needs stable id, kind, workstream, sprint, rationale (strings), inputs "
    "(object), timeout_seconds (positive integer), expected_revision (integer). "
    "These are proposals for the coordinator to validate, never executed actions."
)


class LaunchError(Exception):
    """Only fixed, non-sensitive error codes cross the CLI boundary."""


def require(condition, code):
    if not condition:
        raise LaunchError(code)


def now():
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def no_links(path):
    path = Path(os.path.abspath(path))
    require(all(not p.is_symlink() for p in (path, *path.parents)), "symlink_path")
    return path


def private_dir(path):
    path = no_links(path)
    info = path.stat()
    require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
            and info.st_mode & 0o077 == 0, "unsafe_runs_directory")
    return path


def read_file(path, limit, private=False):
    path = no_links(path)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        info = os.fstat(stream.fileno())
        require(stat.S_ISREG(info.st_mode) and info.st_size <= limit, "invalid_file")
        if private:
            require(info.st_uid == os.getuid() and info.st_mode & 0o077 == 0
                    and info.st_nlink == 1, "unsafe_private_file")
        data = stream.read(limit + 1)
    require(len(data) <= limit, "file_limit")
    return data


def strict_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate_json_key")
            result[key] = value
        return result
    def constant(_):
        raise LaunchError("invalid_json_number")
    try:
        return json.loads(data, object_pairs_hook=pairs, parse_constant=constant)
    except (ValueError, UnicodeError, RecursionError):
        raise LaunchError("invalid_json") from None


def write_file(path, data, mode=0o600):
    """Exclusive artifacts, fsynced before consumers see their rename."""
    data = data.encode() if isinstance(data, str) else data
    temporary = path.with_name(path.name + ".pending")
    fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    with os.fdopen(fd, "wb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)
    fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def write_json(path, value):
    write_file(path, json.dumps(value, ensure_ascii=True, indent=2) + "\n")


def auth_token(path):
    value = strict_json(read_file(path, 16384, private=True))
    require(isinstance(value, dict) and set(value) == {"CLAUDE_CODE_OAUTH_TOKEN"},
            "invalid_auth_contract")
    token = value["CLAUDE_CODE_OAUTH_TOKEN"]
    require(isinstance(token, str) and 16 <= len(token) <= 8192
            and all(33 <= ord(c) <= 126 for c in token), "invalid_auth_contract")
    return token


def redacted(text, token=""):
    if token:
        text = text.replace(token, "[REDACTED]")
    return SECRET.sub("[REDACTED]", text)


def no_secrets(value, token, code):
    """Check decoded JSON too, so Unicode escapes cannot conceal an echoed token."""
    if isinstance(value, str):
        require(redacted(value, token) == value, code)
    elif isinstance(value, dict):
        for key, item in value.items():
            no_secrets(key, token, code)
            no_secrets(item, token, code)
    elif isinstance(value, list):
        for item in value:
            no_secrets(item, token, code)


def read_only_context(p):
    profile = p.get("permission_profile")
    fs = p.get("file_system_sandbox_policy")
    if profile is not None:
        require(profile.get("type") == "managed" and profile.get("network") == "restricted",
                "wrong_permission_profile")
        fs = profile.get("file_system", {})
    if fs is not None:
        require(fs.get("kind", fs.get("type")) == "restricted"
                and all(entry.get("access") != "write" for entry in fs.get("entries", [])),
                "wrong_filesystem_permissions")


def decision(text):
    require(isinstance(text, str) and len(text.encode()) <= FINAL_LIMIT, "invalid_final")
    value = strict_json(text)
    require(isinstance(value, dict) and type(value.get("state_revision")) is int
            and isinstance(value.get("actions"), list), "invalid_decision")
    require(len(value["actions"]) <= 100, "action_limit")
    ids = set()
    for action in value["actions"]:
        require(isinstance(action, dict), "invalid_action")
        for key in ("id", "kind", "workstream", "sprint", "rationale"):
            require(isinstance(action.get(key), str) and bool(action[key].strip()),
                    "invalid_action")
        require(action["id"] not in ids, "duplicate_action_id")
        ids.add(action["id"])
        require(isinstance(action.get("inputs"), dict)
                and type(action.get("expected_revision")) is int
                and type(action.get("timeout_seconds")) is int
                and action["timeout_seconds"] > 0, "invalid_action")
    return value


def evidence(records, cwd):
    """Correlate one fresh session and one turn; never scrape JSON from pane/tools."""
    result = {"session_id": None, "thread_id": None, "turn_id": None, "final": None}
    started, context, response, completed = None, None, None, False
    for record in records:
        require(isinstance(record, dict) and isinstance(record.get("payload"), dict),
                "invalid_record")
        kind, p = record.get("type"), record["payload"]
        event = p.get("type")
        if kind == "session_meta":
            require(result["session_id"] is None and p.get("cwd") == str(cwd)
                    and p.get("model_provider") == PROVIDER, "wrong_session")
            require(not any(p.get(k) for k in ("forked_from_id", "parent_thread_id",
                    "history_base", "subagent_history_start_ordinal")), "inherited_context")
            require(p.get("source") == "cli", "wrong_session_source")
            for key in ("id", "session_id"):
                try:
                    uuid.UUID(p[key])
                except (KeyError, ValueError, TypeError, AttributeError):
                    raise LaunchError("invalid_session_identity") from None
            result.update(session_id=p["session_id"], thread_id=p["id"])
        else:
            require(result["session_id"] is not None, "missing_session_metadata")
        if kind == "turn_context":
            require(not completed and p.get("model") == MODEL
                    and p.get("model_provider") == PROVIDER and p.get("effort") == EFFORT
                    and p.get("cwd") == str(cwd) and p.get("approval_policy") == "never"
                    and p.get("sandbox_policy", {}).get("type") == "read-only",
                    "wrong_turn_configuration")
            read_only_context(p)
            require(isinstance(p.get("turn_id"), str) and bool(p["turn_id"])
                    and context in (None, p["turn_id"]), "multiple_turns")
            context = p["turn_id"]
        if kind == "response_item" and event in (
                "function_call", "custom_tool_call", "local_shell_call", "web_search_call",
                "tool_search_call", "image_generation_call"):
            raise LaunchError("unexpected_tool_call")
        if kind == "event_msg":
            require(event not in ("error", "model_reroute", "turn_aborted")
                    and not p.get("error"), "runtime_failure")
            if event == "item_completed":
                require(p.get("item", {}).get("type") in ("UserMessage", "AgentMessage", "Reasoning"),
                        "unexpected_turn_item")
            if event in ("task_started", "turn_started"):
                require(started is None and not completed and bool(p.get("turn_id")),
                        "multiple_turns")
                started = p["turn_id"]
            elif event == "model_response_completed":
                require(not completed and p.get("turn_id") == started == context
                        and p.get("model") == MODEL and p.get("model_provider_id") == PROVIDER
                        and isinstance(p.get("response_id"), str) and bool(p["response_id"]),
                        "wrong_response_identity")
                require(p.get("finish_reason") in (None, "stop", "end_turn", "stop_sequence"),
                        "incomplete_provider_response")
                response = p["response_id"]
            elif event in ("task_complete", "turn_complete"):
                require(not completed and response is not None
                        and p.get("turn_id") == started == context, "uncorrelated_completion")
                decision(p.get("last_agent_message"))
                result.update(turn_id=context, response_id=response, final=p["last_agent_message"])
                completed = True
    return result


def environment(run):
    """No caller config, proxies, credentials, session IDs, Python hooks or TMUX state."""
    return {"PATH": SAFE_PATH, "HOME": str(run / "user"), "TERM": "xterm-256color",
            "LANG": "en_US.UTF-8", "LC_ALL": "en_US.UTF-8", "RUST_LOG": "trace",
            "CODEX_HOME": str(run / "home"), "CORBANU_HOME": str(run / "home"),
            "PFTERMINAL_HOME": str(run / "home"), "TMPDIR": str(run / "tmp"),
            "GIT_CEILING_DIRECTORIES": str(run),
            "XDG_CONFIG_HOME": str(run / "user/config"),
            "XDG_CACHE_HOME": str(run / "user/cache"),
            "XDG_DATA_HOME": str(run / "user/data"),
            "CLAUDE_CONFIG_DIR": str(run / "user/claude")}


def command(binary, run):
    values = [f'model_provider="{PROVIDER}"', f'model_reasoning_effort="{EFFORT}"',
              "log_dir=" + json.dumps(str(run / "logs")), "check_for_update_on_startup=false",
              "analytics.enabled=false", "tui.animations=false", "web_search=\"disabled\"",
              "project_doc_max_bytes=0", "project_root_markers=[]", "skills.include_instructions=false",
              "skills.bundled.enabled=false", "orchestrator.skills.enabled=false",
              "orchestrator.mcp.enabled=false", "agents.enabled=false",
              "tools.update_plan.enabled=false", "tools.experimental_request_user_input.enabled=false",
              "developer_instructions=" + json.dumps(INSTRUCTIONS)]
    disabled = ("shell_tool unified_exec shell_snapshot code_mode code_mode_host js_repl "
                "multi_agent multi_agent_v2 enable_fanout apps enable_mcp_apps plugins plugin_hooks "
                "hooks recommended_plugins tool_search tool_suggest search_tool browser_use "
                "computer_use image_generation memories external_agent_memory_import "
                "remote_models remote_control in_app_updates request_permissions_tool "
                "skill_search skill_mcp_dependency_install goals token_budget sleep_tool "
                "current_time_reminder workspace_dependencies").split()
    values += [f"features.{key}=false" for key in disabled]
    args = [str(binary), "--no-alt-screen", "--model", MODEL, "--sandbox", "read-only",
            "--ask-for-approval", "never", "-C", str(run / "packet")]
    for value in values:
        args.extend(["-c", value])
    return args


def processes():
    proc = subprocess.run(["/bin/ps", "-axo", "pid=,ppid=,pgid=,stat=,lstart="],
                          capture_output=True, text=True, timeout=3, env={"PATH": SAFE_PATH})
    require(proc.returncode == 0, "process_inspection_failed")
    result = {}
    for line in proc.stdout.splitlines():
        fields = line.split(None, 4)
        if len(fields) == 5:
            pid, parent, group = map(int, fields[:3])
            result[pid] = (parent, group, fields[3], fields[4])
    return result


def child(run):
    """TMUX exec helper; the credential is injected only into the binary environment."""
    os.umask(0o077)
    try:
        params = strict_json(read_file(run / "launch.json", 65536, private=True))
        require(file_digest(Path(params["binary"])) == params["binary_sha256"], "binary_changed")
        token = auth_token(Path(params["auth_file"]))
        require(digest(token.encode()) == params["auth_sha256"], "auth_file_changed")
        env = environment(run)
        env["CLAUDE_CODE_OAUTH_TOKEN"] = token
        if os.getpgrp() != os.getpid():
            os.setpgid(0, 0)
        info = processes()[os.getpid()]
        write_json(run / "process.json", {"pid": os.getpid(), "pgid": os.getpgrp(),
                                         "started": info[3]})
        # Parent opens this gate only after recording ownership. Cancellation before
        # gate creation cannot start the authenticated application.
        deadline = time.monotonic() + 15
        while not (run / "go").exists():
            require(time.monotonic() < deadline and not (run / "stop").exists(), "start_cancelled")
            time.sleep(0.05)
        require(not (run / "stop").exists(), "start_cancelled")
        os.chdir(run / "packet")
        os.execve(params["binary"], params["argv"], env)
    except BaseException:
        # Never emit an exception containing credential contents or environment.
        write_json(run / "child-error.json", {"error": "child_start_failed"})
        return 1


class Tui:
    def __init__(self, run, tmux):
        self.run, self.tmux_bin = run, tmux
        self.socket, self.session = str(run / "tmux.sock"), "manager"
        require(len(self.socket.encode()) < 100, "tmux_socket_path_too_long")
        self.owned, self.pane, self.launched = {}, None, False
        self.events = []

    def tmux(self, *args, check=True):
        result = subprocess.run([self.tmux_bin, "-S", self.socket, *args],
                                env=environment(self.run), cwd=self.run / "packet",
                                capture_output=True, text=True, timeout=3)
        require(not check or result.returncode == 0, "tmux_command_failed")
        return result

    def start(self):
        self.launched = True  # A failed command may still have created the session.
        self.pane = self.tmux("-f", "/dev/null", "new-session", "-d", "-P", "-F", "#{pane_id}",
                              "-s", self.session, "-x", "150", "-y", "46", "-c",
                              str(self.run / "packet"),
                              "exec /bin/sh " + shlex.quote(str(self.run / "launch.sh"))).stdout.strip()
        require(re.fullmatch(r"%\d+", self.pane) is not None, "invalid_tmux_pane")
        self.tmux("set-option", "-w", "-t", self.pane, "remain-on-exit", "on")

    def capture(self):
        return self.tmux("capture-pane", "-p", "-t", self.session, "-S", "-200", check=False).stdout

    def send(self, text):
        for index in range(0, len(text), 4000):
            self.tmux("send-keys", "-t", self.session, "-l", "--", text[index:index + 4000])
        self.events.append({"at": now(), "text_sha256": digest(text.encode())})
        # Bracketed paste is unnecessary for this bounded single-line JSON prompt.
        time.sleep(0.3)
        self.tmux("send-keys", "-t", self.session, "Enter")
        self.events.append({"at": now(), "key": "Enter"})

    def observe(self):
        table = processes()
        owner_file = self.run / "process.json"
        if owner_file.exists():
            owner = strict_json(read_file(owner_file, 4096, private=True))
            pid = owner["pid"]
            require(type(pid) is int and pid > 1 and owner["pgid"] == pid, "invalid_owned_process")
            if pid in table and table[pid][3] == owner["started"]:
                self.owned[pid] = owner["started"]
        changed = True
        while changed:
            changed = False
            for pid, (parent, group, _, started) in table.items():
                if pid in self.owned:
                    continue
                if any(ref in self.owned and table.get(ref, (None,) * 4)[3] == self.owned[ref]
                       for ref in (parent, group)):
                    self.owned[pid] = started
                    changed = True
        return {pid for pid, start in self.owned.items() if pid in table
                and table[pid][3] == start and not table[pid][2].startswith("Z")}

    def stop(self):
        write_file(self.run / "stop", "stop\n")
        errors, forced = [], False
        if not self.launched:
            return {"clean": True, "forced": False, "errors": []}
        try:
            self.observe()
            self.tmux("send-keys", "-t", self.session, "Escape", check=False)
            self.tmux("send-keys", "-t", self.session, "C-u", check=False)
            self.send("/exit")
        except Exception:
            errors.append("normal_exit_unavailable")
        try:
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                if not self.observe():
                    break
                time.sleep(0.1)
            for sig in (signal.SIGTERM, signal.SIGKILL):
                alive = self.observe()
                forced |= bool(alive)
                for pid in sorted(alive, reverse=True):
                    try:
                        # Recheck identity immediately before signalling; no name-based kills.
                        if pid in self.observe():
                            os.kill(pid, sig)
                    except ProcessLookupError:
                        pass
                deadline = time.monotonic() + 1
                while time.monotonic() < deadline and self.observe():
                    time.sleep(0.05)
        finally:
            # Even an inspection/signal exception must attempt the exact owned session.
            self.tmux("kill-session", "-t", self.session, check=False)
        probe = self.tmux("list-sessions", check=False)
        # The socket was exclusively created here with no user tmux config.
        session_gone = probe.returncode != 0 and not probe.stdout.strip()
        return {"clean": session_gone and not self.observe(), "forced": forced,
                "session_gone": session_gone, "errors": errors,
                "owned_pids": sorted(self.owned)}


def rollout(run):
    paths = sorted((run / "home/sessions").rglob("*.jsonl"))
    require(len(paths) <= 1, "multiple_session_files")
    if not paths:
        return None, []
    raw = read_file(paths[0], RECORD_LIMIT, private=True)
    # An unfinished line cannot be a completion record.
    lines = raw.split(b"\n")[:-1]
    return paths[0], [strict_json(line) for line in lines if line]


def run_launcher(args):
    started = now()
    receipt = {"run_id": uuid.uuid4().hex, "status": "failed", "model": MODEL,
               "provider": PROVIDER, "effort": EFFORT, "session_id": None,
               "decision": None, "artifacts": {}, "started_at": started}
    run, tui, token = None, None, ""
    previous_handlers = {}
    old_umask = os.umask(0o077)
    try:
        require(math.isfinite(args.timeout) and 1 <= args.timeout <= 3600, "invalid_timeout")
        root = private_dir(args.runs_dir)
        run = Path(tempfile.mkdtemp(prefix="f-", dir=root))
        receipt["run_id"] = run.name
        receipt["artifacts"] = {"run_dir": str(run), "receipt": str(run / "receipt.json"),
                                "manifest": str(run / "manifest.json"),
                                "private_logs": str(run / "logs")}
        write_json(run / "receipt.json", receipt)
        def cancelled(signum, _frame):
            raise LaunchError("cancelled")
        for sig in (signal.SIGTERM, signal.SIGINT):
            previous_handlers[sig] = signal.signal(sig, cancelled)
        for directory in ("home", "user", "tmp", "packet", "logs"):
            (run / directory).mkdir(mode=0o700)
        token = auth_token(args.auth_file)
        brief_raw = read_file(args.briefing, BRIEF_LIMIT)
        brief = strict_json(brief_raw)
        require(isinstance(brief, dict), "invalid_briefing")
        brief_text = json.dumps(brief, ensure_ascii=True, separators=(",", ":"))
        no_secrets(brief, token, "secret_in_briefing")
        binary = no_links(args.binary)
        require(binary.is_file() and os.access(binary, os.X_OK), "invalid_binary")
        binary_hash = file_digest(binary)
        version = subprocess.run([str(binary), "--version"], env=environment(run),
                                 cwd=run / "packet", capture_output=True, text=True, timeout=5)
        require(version.returncode == 0 and len(version.stdout) <= 512, "binary_version_failed")
        argv = command(binary, run)
        # Only this fresh neutral packet is trusted; no ancestor/project docs are loaded.
        write_file(run / "home/config.toml", "[projects." + json.dumps(str(run / "packet"))
                   + ']\ntrust_level = "trusted"\n')
        write_file(run / "packet/briefing.md", INSTRUCTIONS + "\n\n" + brief_text + "\n")
        source = Path(__file__).resolve()
        write_file(run / "launcher.py", source.read_bytes())
        launch = "#!/bin/sh\nexec " + shlex.join([sys.executable, "-I", str(run / "launcher.py"),
                                                 "--child", str(run)]) + "\n"
        write_file(run / "launch.sh", launch, 0o700)
        write_json(run / "launch.json", {"binary": str(binary), "binary_sha256": binary_hash,
                                         "auth_file": str(no_links(args.auth_file)),
                                         "auth_sha256": digest(token.encode()), "argv": argv})
        manifest = {"run_id": run.name, "started_at": started, "briefing_sha256": digest(brief_raw),
                    "packet_sha256": file_digest(run / "packet/briefing.md"),
                    "launcher_sha256": file_digest(run / "launcher.py"),
                    "launch_sh_sha256": digest(launch.encode()), "binary": str(binary),
                    "binary_version": redacted(version.stdout.strip(), token),
                    "binary_sha256": binary_hash, "argv": argv, "briefing": brief,
                    "model": MODEL, "provider": PROVIDER, "effort": EFFORT}
        write_json(run / "manifest.json", manifest)
        tmux = shutil.which("tmux", path=SAFE_PATH)
        require(tmux is not None, "tmux_missing")
        tui = Tui(run, tmux)
        receipt.update(tmux_socket=tui.socket, tmux_session=tui.session)
        deadline = time.monotonic() + args.timeout
        tui.start()
        sent, gate = False, False
        while time.monotonic() < deadline:
            alive = tui.observe()
            if (run / "child-error.json").exists():
                raise LaunchError("child_start_failed")
            if alive and not gate:
                write_file(run / "go", "go\n")
                gate = True
            require(not gate or bool(alive), "application_exited")
            path, records = rollout(run)
            state = evidence(records, run / "packet")
            receipt.update({k: v for k, v in state.items() if k != "final"})
            pane = tui.capture()
            if not sent and "Corbanu Terminal" in pane and "tok/s" in pane:
                require(state["final"] is None and not any(
                    r["type"] in ("turn_context", "response_item") for r in records), "preexisting_turn")
                write_file(run / "ready-pane.txt", redacted(pane, token))
                tui.send(INSTRUCTIONS + " Briefing JSON: " + brief_text)
                sent = True
            if state["final"] is not None:
                require(sent, "unsolicited_completion")
                require(redacted(state["final"], token) == state["final"], "secret_in_decision")
                receipt["decision"] = decision(state["final"])
                no_secrets(receipt["decision"], token, "secret_in_decision")
                write_file(run / "final.txt", state["final"])
                write_file(run / "final-pane.txt", redacted(pane, token))
                receipt["artifacts"].update(final=str(run / "final.txt"), private_rollout=str(path))
                receipt["status"] = "completed"
                # Candidate answer is durable before shutdown, but not yet dispatchable.
                write_json(run / "candidate.json", {**receipt, "status": "pending_shutdown"})
                break
            time.sleep(0.1)
        else:
            raise LaunchError("timeout")
    except BaseException as exc:
        code = str(exc) if isinstance(exc, LaunchError) else "launcher_failure"
        if isinstance(exc, KeyboardInterrupt):
            code = "cancelled"
        receipt.update(error=code, status="timeout" if code == "timeout" else "failed",
                       decision=None)
    finally:
        # A repeated cancellation must not interrupt owned-process cleanup.
        for sig in previous_handlers:
            signal.signal(sig, signal.SIG_IGN)
        try:
            if tui is not None:
                try:
                    receipt["shutdown"] = tui.stop()
                except BaseException:
                    receipt["shutdown"] = {"clean": False, "errors": ["shutdown_verification_failed"]}
                write_json(run / "keys.json", tui.events)
            else:
                receipt["shutdown"] = {"clean": True, "launched": False}
            if not receipt["shutdown"]["clean"]:
                receipt.update(status="failed", error="shutdown_failed", decision=None)
            if receipt["status"] == "completed":
                try:
                    _, records = rollout(run)
                    final_state = evidence(records, run / "packet")
                    require(final_state["session_id"] == receipt["session_id"]
                            and final_state["turn_id"] == receipt["turn_id"]
                            and decision(final_state["final"]) == receipt["decision"],
                            "final_evidence_changed")
                except Exception:
                    receipt.update(status="failed", error="final_evidence_invalid", decision=None)
            receipt["finished_at"] = now()
            # Fixed errors and selected metadata only; raw traces are private, never exported.
            receipt = strict_json(redacted(json.dumps(receipt), token))
            if run is not None:
                write_json(run / "receipt.json", receipt)
        except BaseException:
            receipt.update(status="failed", error="receipt_write_failed", decision=None)
            # Initial failed receipt remains durable if the filesystem cannot accept writes.
        finally:
            for sig, handler in previous_handlers.items():
                signal.signal(sig, handler)
            os.umask(old_umask)
    return receipt


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("briefing", "runs-dir", "binary", "auth-file"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=300)
    args = parser.parse_args(argv)
    receipt = run_launcher(args)
    print(json.dumps(receipt, ensure_ascii=True))
    return 0 if receipt["status"] == "completed" else 1


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--child":
        sys.exit(child(Path(sys.argv[2])))
    sys.exit(main())

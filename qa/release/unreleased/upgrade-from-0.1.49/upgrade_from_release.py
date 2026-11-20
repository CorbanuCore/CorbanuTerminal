#!/usr/bin/env python3
"""Released 0.1.49 home -> sync candidate upgrade, then backup-and-restore rollback.

Runs inside an isolated container (no network, no host home, no keyring) with
/work mounted. Phases, each its own invocation so a failure stops the run:

  baseline  old binary: config, two exec sessions, a TUI session with a vault
            credential, named accounts, API-key login, an archived session and
            memory files; then a stopped-home backup (the release note's step).
  upgrade   candidate on the same home: every check, a resumed session, a
            second start, and a schema comparison with a fresh candidate home.
  downgrade old binary on a copy of the upgraded home: expected refusal.
  rollback  restore the backup, old binary: sessions, vault, accounts, DBs.
  summarize write the committed results/<phase>.json from evidence/, after
            checking that no fixture credential value appears in them.

Every credential is a synthetic fixture string generated here; values are only
compared in memory, never printed. Raw results go to /work/evidence/<phase>.json
(with TUI screen captures; the vault value is typed into the TUI, so keep
evidence/ out of the repository).
"""
import hashlib
import json
import os
import pathlib
import secrets
import shlex
import shutil
import sqlite3
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = pathlib.Path("/work")
PHASE = sys.argv[1]
OS_HOME = ROOT / "os-home"
HOME = OS_HOME / ".corbanu"
EVIDENCE = ROOT / "evidence"
WORKSPACE = ROOT / "workspace"
BACKUP = ROOT / "backup-0.1.49"
FIXTURE = ROOT / "fixture-values.json"
OLD = "/work/old/bin/corbanu"
NEW = "/work/new/bin/corbanu"
PORT = 18750
for path in (OS_HOME, EVIDENCE, WORKSPACE):
    path.mkdir(exist_ok=True)
# Nothing of the host may be visible: no homes, user sockets or Docker socket.
HOST_STATE = (".ssh", ".corbanu", ".codex", ".config/gh", ".local/share/keyrings")
for leak in [
    *(home / name for home in [*pathlib.Path("/home").glob("*"), pathlib.Path("/root")] for name in HOST_STATE),
    pathlib.Path("/run/user"),
    pathlib.Path("/var/run/docker.sock"),
]:
    assert not os.path.lexists(leak), leak  # unreadable counts as absent

ENV = dict(
    os.environ,
    HOME=str(OS_HOME),
    CORBANU_TEST_NO_NATIVE_KEYRING="1",
    TERM="xterm-256color",
    XDG_CONFIG_HOME=str(OS_HOME / ".config"),
    XDG_DATA_HOME=str(OS_HOME / ".local/share"),
    XDG_CACHE_HOME=str(OS_HOME / ".cache"),
    MOCK_API_KEY="fixture-mock-key",
)
for name in ("CORBANU_HOME", "CODEX_HOME", "PFTERMINAL_HOME", "DBUS_SESSION_BUS_ADDRESS"):
    ENV.pop(name, None)

REQUESTS = []


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"data":[],"models":[]}')

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        REQUESTS.append({"path": self.path, "model": body.get("model")})
        n = len(REQUESTS)
        message = {
            "type": "message",
            "id": f"msg_{n}",
            "role": "assistant",
            "status": "completed",
            "content": [{"type": "output_text", "text": f"UPGRADE_REPLY_{PHASE}_{n}", "annotations": []}],
        }
        response = {
            "id": f"resp_{n}",
            "object": "response",
            "status": "completed",
            "model": body.get("model"),
            "output": [message],
            "usage": {"input_tokens": 20, "output_tokens": 5, "total_tokens": 25},
        }
        events = [
            ("response.created", {"response": dict(response, status="in_progress", output=[])}),
            ("response.output_item.done", {"output_index": 0, "item": message}),
            ("response.completed", {"response": response}),
        ]
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        for name, payload in events:
            self.wfile.write(f"event: {name}\ndata: {json.dumps(dict(payload, type=name))}\n\n".encode())
            self.wfile.flush()


SERVER = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
threading.Thread(target=SERVER.serve_forever, daemon=True).start()

CONFIG = f'''model = "gpt-4.1"
model_provider = "release-fixture"
approval_policy = "never"
sandbox_mode = "read-only"
check_for_update_on_startup = false

[features]
named_accounts = true

[model_providers.release-fixture]
name = "Isolated release fixture"
base_url = "http://127.0.0.1:{PORT}/v1"
wire_api = "responses"
env_key = "MOCK_API_KEY"

[projects."{WORKSPACE}"]
trust_level = "trusted"
'''

MEMORY_FILES = {
    "memories/MEMORY.md": "# Memory\n\n- Fixture: the user prefers concise answers.\n",
    "memories/memory_summary.md": "Fixture memory summary written for the 0.1.49 upgrade check.\n",
}
ACCOUNTS = [("zai", "personal", "api-key"), ("anthropic", "work", "api-key"), ("claude-plan", "team", "claude-token")]

RESULT = {"phase": PHASE, "checks": {}, "observations": {}}


def check(name, ok, detail=None):
    RESULT["checks"][name] = bool(ok)
    if detail is not None:
        RESULT["observations"][name] = detail


def run(binary, *args, stdin=None, timeout=120):
    proc = subprocess.run(
        [binary, *args],
        input=stdin,
        capture_output=True,
        text=True,
        env=ENV,
        cwd=WORKSPACE,
        timeout=timeout,
    )
    return proc


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_digest(root):
    """sha256 of every regular file below root, relative names."""
    return {
        str(p.relative_to(root)): sha(p)
        for p in sorted(root.rglob("*"))
        if p.is_file()
        and not p.name.endswith((".sqlite-wal", ".sqlite-shm", ".lock"))
        and not str(p.relative_to(root)).startswith("tmp/")
    }


def databases():
    out = {}
    for db in sorted(HOME.glob("*.sqlite")):
        con = sqlite3.connect(db)
        con.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        info = {"integrity": con.execute("PRAGMA integrity_check").fetchone()[0]}
        tables = {r[0] for r in con.execute("select name from sqlite_master where type='table'")}
        if "_sqlx_migrations" in tables:
            info["migrations"] = [
                [v, d, c.hex(), s]
                for v, d, c, s in con.execute(
                    "select version, description, checksum, success from _sqlx_migrations order by installed_on, version"
                )
            ]
        if "threads" in tables:
            info["threads"] = [
                list(r)
                for r in con.execute(
                    "select id, title, archived, rollout_path, model_provider from threads order by id"
                )
            ]
        if "provider_request_state" in tables:
            info["provider_request_state_columns"] = [
                r[1] for r in con.execute("pragma table_info(provider_request_state)")
            ]
        if "stage1_outputs" in tables:
            info["stage1_outputs"] = con.execute("select count(*) from stage1_outputs").fetchone()[0]
        out[db.name] = info
        con.close()
    return out


def schema(db):
    con = sqlite3.connect(db)
    rows = con.execute(
        "select type, name, sql from sqlite_master where name != '_sqlx_migrations' order by type, name"
    ).fetchall()
    con.close()
    return rows


def thread_ids():
    con = sqlite3.connect(HOME / "pfterminal_state_5.sqlite")
    rows = con.execute("select id, archived from threads order by created_at_ms, id").fetchall()
    con.close()
    return rows


def rollout_for(thread_id):
    hits = [p for p in HOME.rglob(f"rollout-*{thread_id}.jsonl")]
    return hits[0] if len(hits) == 1 else None


def tmux_session(binary, args, steps, label, os_home=OS_HOME):
    """Drive the TUI in tmux; steps are (send_text_or_None, wait_needle).

    Returns the startup capture."""
    sock = f"/tmp/{PHASE}-{label}.sock"
    conf = ROOT / "tmux.conf"
    conf.write_text("set-option -g remain-on-exit on\n")

    def tm(*a, check_rc=True):
        return subprocess.run(
            ["tmux", "-f", str(conf), "-S", sock, *a], check=check_rc, capture_output=True, text=True, env=ENV
        ).stdout

    cmd = ["env", "CORBANU_TEST_NO_NATIVE_KEYRING=1", "MOCK_API_KEY=fixture-mock-key", f"HOME={os_home}",
           binary, "--no-alt-screen", "-C", str(WORKSPACE), *args]
    tm("new-session", "-d", "-s", "qa", "-x", "140", "-y", "45", shlex.join(cmd))

    def capture(name):
        text = tm("capture-pane", "-t", "qa", "-p", "-S", "-2000", check_rc=False)
        (EVIDENCE / f"{PHASE}-{label}-{name}.txt").write_text(text)
        return text

    def wait(needle, name, timeout=45):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            text = capture(name)
            if needle in text:
                return text
            time.sleep(0.3)
        raise RuntimeError(f"{label}: checkpoint absent: {needle!r} (capture {name})")

    try:
        time.sleep(3)
        start = capture("startup")
        if "trust" in start.lower() and "directory" in start.lower():
            tm("send-keys", "-t", "qa", "Enter")
            time.sleep(2)
        for i, (text, needle) in enumerate(steps):
            if text is not None:
                tm("send-keys", "-t", "qa", "-l", text)
                time.sleep(0.2)
                tm("send-keys", "-t", "qa", "Enter")
                time.sleep(0.5)
            if needle is not None:
                wait(needle, f"step{i}")
        tm("send-keys", "-t", "qa", "-l", "/exit")
        time.sleep(0.2)
        tm("send-keys", "-t", "qa", "Enter")
        time.sleep(2)
        capture("exit")
        return start
    finally:
        tm("kill-session", "-t", "qa", check_rc=False)


def credential_checks(binary, values, prefix):
    """Vault credential, named accounts and API-key login, compared in memory."""
    reveal = run(binary, "vault", "auth-helper", "upgrade-canary")
    check(f"{prefix}vault_credential_readable", reveal.returncode == 0 and reveal.stdout.strip() == values["vault"],
          {"exit": reveal.returncode, "stderr_tail": reveal.stderr.strip().splitlines()[-1:] if reveal.returncode else []})
    listing = run(binary, "account", "list")
    check(f"{prefix}named_accounts_listed",
          listing.returncode == 0 and all(f"{p}\t{n}\t" in listing.stdout for p, n, _ in ACCOUNTS),
          {"exit": listing.returncode, "stdout": listing.stdout.strip().splitlines()})
    for value in values.values():
        assert value not in listing.stdout
    status = run(binary, "login", "status")
    check(f"{prefix}api_key_login_present",
          status.returncode == 0 and "Logged in using an API key" in (status.stdout + status.stderr),
          {"exit": status.returncode, "text": (status.stdout + status.stderr).strip().splitlines()[-1:]})
    for value in values.values():
        assert value not in status.stdout + status.stderr


def secret_scan(values):
    """No synthetic credential value appears in clear text anywhere in the home."""
    # The API-key login lives in auth.json by design (file credential store);
    # only vault-held values must never appear in clear text.
    hits = []
    needles = [v.encode() for k, v in values.items() if k != "login"]
    for path in HOME.rglob("*"):
        if path.is_file():
            data = path.read_bytes()
            if any(n in data for n in needles):
                hits.append(str(path.relative_to(HOME)))
    return hits


def baseline():
    assert not HOME.exists(), "baseline already exists"
    HOME.mkdir(parents=True)
    (HOME / "config.toml").write_text(CONFIG)
    values = {
        "vault": "fixture-vault-" + secrets.token_urlsafe(24),
        "zai/personal": "fixture-zai-" + secrets.token_urlsafe(24),
        "anthropic/work": "sk-ant-fixture-" + secrets.token_urlsafe(24),
        "claude-plan/team": "sk-ant-oat-fixture-" + secrets.token_urlsafe(24),
        "login": "sk-fixture-login-" + secrets.token_urlsafe(24),
    }
    FIXTURE.write_text(json.dumps(values))
    FIXTURE.chmod(0o600)
    RESULT["version"] = run(OLD, "--version").stdout.strip()
    RESULT["binary_sha256"] = sha(pathlib.Path(OLD).resolve())

    # Two non-interactive sessions with rollouts.
    for n in (1, 2):
        proc = run(OLD, "exec", "--skip-git-repo-check", f"Reply for baseline exec {n}")
        check(f"exec_{n}_completed", proc.returncode == 0 and "UPGRADE_REPLY_baseline" in proc.stdout,
              {"exit": proc.returncode})
    # An interactive session that adds an operational vault credential.
    tmux_session(OLD, [], [
        ("/vault credential add", "Label"),
        ("upgrade-canary", None),
        (values["vault"], "Added vault credential"),
        ("Reply for baseline tui", "UPGRADE_REPLY_baseline"),
    ], "tui")
    for provider, name, kind in ACCOUNTS:
        proc = run(OLD, "account", "add", provider, name, "--kind", kind, stdin=values[f"{provider}/{name}"])
        check(f"account_add_{provider}_{name}", proc.returncode == 0,
              {"exit": proc.returncode, "stderr_tail": proc.stderr.strip().splitlines()[-1:]})
    proc = run(OLD, "login", "--with-api-key", stdin=values["login"])
    check("api_key_login", proc.returncode == 0, {"exit": proc.returncode})
    # Archive the first session.
    threads = thread_ids()
    check("three_sessions", len(threads) == 3, threads)
    proc = run(OLD, "archive", threads[0][0])
    check("archive_session", proc.returncode == 0, {"exit": proc.returncode, "stderr": proc.stderr.strip().splitlines()[-2:]})
    for rel, text in MEMORY_FILES.items():
        (HOME / rel).parent.mkdir(parents=True, exist_ok=True)
        (HOME / rel).write_text(text)
    credential_checks(OLD, values, "")
    RESULT["databases"] = databases()
    RESULT["threads"] = thread_ids()
    RESULT["files"] = tree_digest(HOME)
    RESULT["secret_scan_hits"] = secret_scan(values)
    check("no_clear_text_credentials", not RESULT["secret_scan_hits"])
    RESULT["requests"] = len(REQUESTS)
    # Release note: back up the complete home with Corbanu stopped.
    shutil.copytree(HOME, BACKUP, symlinks=True)
    RESULT["backup_files"] = len(tree_digest(BACKUP))


def upgrade():
    values = json.loads(FIXTURE.read_text())
    base = json.loads((EVIDENCE / "baseline.json").read_text())
    RESULT["version"] = run(NEW, "--version").stdout.strip()
    RESULT["binary_sha256"] = sha(pathlib.Path(NEW).resolve())
    before_files = base["files"]

    # First candidate start: a resumed exec on the newest (unarchived) session.
    threads = thread_ids()
    live = [t for t, archived in threads if not archived]
    for n, target in enumerate(live):
        prefix = rollout_for(target).read_bytes()
        proc = run(NEW, "exec", "--skip-git-repo-check", "resume", target, "Reply after upgrade")
        check(f"resume_exec_{n}_completed", proc.returncode == 0 and "UPGRADE_REPLY_upgrade" in proc.stdout,
              {"exit": proc.returncode})
        after = rollout_for(target).read_bytes()
        check(f"resumed_rollout_{n}_extends_old_prefix", after.startswith(prefix) and len(after) > len(prefix))
    # Interactive resume of the TUI session through the picker-free path.
    # The old transcript is on screen before the new turn.
    tmux_session(NEW, ["resume", "--last"], [
        (None, "Reply for baseline tui"),
        ("Reply in upgraded tui", "UPGRADE_REPLY_upgrade"),
    ], "tui")

    dbs = databases()
    RESULT["databases"] = dbs
    old_state = {m[0]: m for m in base["databases"]["pfterminal_state_5.sqlite"]["migrations"]}
    new_state = dbs["pfterminal_state_5.sqlite"]["migrations"]
    by_version = {m[0]: m for m in new_state}
    check("state_applied_rows_unchanged", all(by_version.get(v) == row for v, row in old_state.items()))
    added = [m for m in new_state if m[0] not in old_state]
    check("state_added_exactly_0054_0069", [m[0] for m in added] == list(range(54, 70)) and all(m[3] for m in added),
          [m[:2] for m in added])
    check("state_0070_applied_before_0054_0069",
          [m[0] for m in new_state][-17:] == [70] + list(range(54, 70)))
    for name, info in dbs.items():
        old = base["databases"].get(name)
        if old and "migrations" in old:
            old_rows = {m[0]: m for m in old["migrations"]}
            new_rows = {m[0]: m for m in info.get("migrations", [])}
            check(f"{name}_applied_rows_unchanged", all(new_rows.get(v) == r for v, r in old_rows.items()))
        check(f"{name}_integrity_ok", info["integrity"] == "ok")
    check("threads_survive", [t[0] for t in thread_ids()] == [t[0] for t in base["threads"]]
          and [t[1] for t in thread_ids()] == [t[1] for t in base["threads"]])
    check("archived_rollout_kept", all(
        (HOME / rel).exists() for rel in before_files if rel.startswith("archived_sessions/")
    ) and any(rel.startswith("archived_sessions/") for rel in before_files))
    changed_old_files = sorted(
        rel for rel, digest in before_files.items()
        if (rel.startswith(("secrets/", "memories/", "archived_sessions/")) or rel == "auth.json")
        and ((HOME / rel).exists() is False or sha(HOME / rel) != digest)
    )
    check("vault_auth_memory_archive_files_byte_identical", not changed_old_files, changed_old_files)
    credential_checks(NEW, values, "")

    # A second start has nothing left to migrate.
    snapshot = json.dumps(databases(), sort_keys=True)
    proc = run(NEW, "exec", "--skip-git-repo-check", "Second start after upgrade")
    check("second_start_completed", proc.returncode == 0, {"exit": proc.returncode})
    second = databases()
    check("second_start_adds_no_migration", all(
        [m[0] for m in second[n].get("migrations", [])] == [m[0] for m in json.loads(snapshot)[n].get("migrations", [])]
        for n in second if n in json.loads(snapshot)
    ))

    # The upgraded schema equals a fresh candidate home's, for every database.
    fresh_os = ROOT / "fresh-home"
    fresh_os.mkdir(exist_ok=True)
    (fresh_os / ".corbanu").mkdir(exist_ok=True)
    (fresh_os / ".corbanu" / "config.toml").write_text(CONFIG)
    env = dict(ENV, HOME=str(fresh_os))
    proc = subprocess.run([NEW, "exec", "--skip-git-repo-check", "fresh"], env=env, cwd=WORKSPACE,
                          capture_output=True, text=True, timeout=120)
    upgraded_dbs = sorted(db.name for db in HOME.glob("*.sqlite"))
    fresh_dbs = sorted(db.name for db in (fresh_os / ".corbanu").glob("*.sqlite"))
    check("fresh_candidate_home_has_same_databases", proc.returncode == 0 and upgraded_dbs == fresh_dbs,
          {"exit": proc.returncode, "upgraded": upgraded_dbs, "fresh": fresh_dbs})
    diffs = {}
    for name in upgraded_dbs:
        fresh_db = fresh_os / ".corbanu" / name
        if not fresh_db.exists() or schema(HOME / name) != schema(fresh_db):
            diffs[name] = sorted(set(map(tuple, schema(HOME / name))) ^ set(map(tuple, schema(fresh_db))))
    check("schemas_match_fresh_candidate_home", not diffs, {"compared": upgraded_dbs, "differences": diffs})
    RESULT["secret_scan_hits"] = secret_scan(values)
    check("no_clear_text_credentials", not RESULT["secret_scan_hits"])
    # Config: every original line kept; record what the candidate added.
    old_lines = (BACKUP / "config.toml").read_text().splitlines()
    new_lines = (HOME / "config.toml").read_text().splitlines()
    added = [line for line in new_lines if line not in old_lines]
    check("config_original_lines_kept", all(line in new_lines for line in old_lines),
          {"added_lines": [line for line in added if line.strip()]})
    RESULT["requests"] = len(REQUESTS)


def downgrade():
    """0.1.49 on a copy of the upgraded home (not a supported path)."""
    copy = ROOT / "downgrade-os-home"
    shutil.copytree(OS_HOME, copy, symlinks=True)
    state = copy / ".corbanu" / "pfterminal_state_5.sqlite"

    def state_rows():
        con = sqlite3.connect(state)
        rows = (
            [r[0] for r in con.execute("select version from _sqlx_migrations order by version")],
            con.execute("select count(*) from threads").fetchone()[0],
        )
        con.close()
        return rows

    versions_before, threads_before = state_rows()
    RESULT["version"] = run(OLD, "--version").stdout.strip()
    # Interactive start: the refusal the user sees.
    startup = tmux_session(OLD, [], [], "tui", os_home=copy)
    flat = "".join(startup.split())
    check("tui_reports_foreign_migrations",
          "appears to be damaged" in startup
          and "differentCodex/CorbanuTerminaldistribution" in flat
          and all(f"version{v}(checksum" in flat for v in range(54, 70))
          and "version70(checksum" not in flat)
    # Non-interactive start.
    env = dict(ENV, HOME=str(copy), RUST_LOG="warn")
    proc = subprocess.run([OLD, "exec", "--skip-git-repo-check", "Reply after downgrade"], env=env, cwd=WORKSPACE,
                          capture_output=True, text=True, timeout=120)
    text = proc.stdout + proc.stderr
    (EVIDENCE / "downgrade-exec-output.txt").write_text(text)
    RESULT["observations"]["exec"] = {
        "exit": proc.returncode,
        "completed_turn": "UPGRADE_REPLY_downgrade" in proc.stdout,
        "mentions_foreign_migrations": "different Codex/Corbanu Terminal distribution" in text,
    }
    versions_after, threads_after = state_rows()
    check("state_db_refused_and_unmodified", versions_after == versions_before == list(range(1, 71))
          and threads_after == threads_before, {"threads": [threads_before, threads_after]})


def rollback():
    values = json.loads(FIXTURE.read_text())
    base = json.loads((EVIDENCE / "baseline.json").read_text())
    upgraded = ROOT / "upgraded-home-kept"
    HOME.rename(upgraded)
    shutil.copytree(BACKUP, HOME, symlinks=True)
    check("restored_files_match_baseline", tree_digest(HOME) == base["files"])
    RESULT["version"] = run(OLD, "--version").stdout.strip()
    threads = thread_ids()
    for n, target in enumerate(t for t, archived in threads if not archived):
        proc = run(OLD, "exec", "--skip-git-repo-check", "resume", target, "Reply after rollback")
        check(f"old_binary_resumes_restored_session_{n}",
              proc.returncode == 0 and "UPGRADE_REPLY_rollback" in proc.stdout, {"exit": proc.returncode})
    tmux_session(OLD, ["resume", "--last"], [("Reply in restored tui", "UPGRADE_REPLY_rollback")], "tui")
    dbs = databases()
    check("restored_state_migrations_are_0_1_49s",
          [m[0] for m in dbs["pfterminal_state_5.sqlite"]["migrations"]] == list(range(1, 54)) + [70])
    check("restored_dbs_integrity_ok", all(i["integrity"] == "ok" for i in dbs.values()))
    credential_checks(OLD, values, "")
    check("archived_session_still_archived", [a for _, a in thread_ids()] == [a for _, a in base["threads"]])


def summarize():
    """Committed results: raw phase results minus the bulk file list."""
    values = json.loads(FIXTURE.read_text()).values()
    out = ROOT / "results"
    out.mkdir(exist_ok=True)
    keep = ("secrets/", "memories/", "archived_sessions/", "sessions/")
    for phase in ("baseline", "upgrade", "downgrade", "rollback"):
        raw = json.loads((EVIDENCE / f"{phase}.json").read_text())
        files = raw.pop("files", None)
        if files is not None:
            raw["file_count"] = len(files)
            raw["files_checked"] = {
                k: v for k, v in files.items() if k.startswith(keep) or k in ("auth.json", "config.toml", "history.jsonl")
            }
        for db in raw.get("databases", {}).values():
            for row in db.get("migrations", []):
                row[2] = row[2][:16]  # checksum prefix
        text = json.dumps(raw, indent=1) + "\n"
        assert not any(value in text for value in values), phase
        (out / f"{phase}.json").write_text(text)
        check(f"{phase}_passed", raw["checks"] and all(raw["checks"].values()) and not raw.get("error"))


try:
    {"baseline": baseline, "upgrade": upgrade, "downgrade": downgrade, "rollback": rollback,
     "summarize": summarize}[PHASE]()
except Exception as exc:  # recorded, then the phase fails
    RESULT["error"] = f"{type(exc).__name__}: {exc}"
finally:
    SERVER.shutdown()
    if PHASE != "summarize":
        (EVIDENCE / f"{PHASE}.json").write_text(json.dumps(RESULT, indent=2) + "\n")

failed = [k for k, v in RESULT["checks"].items() if not v]
print(json.dumps({"phase": PHASE, "version": RESULT.get("version"), "failed": failed, "error": RESULT.get("error")}))
sys.exit(1 if failed or RESULT.get("error") else 0)

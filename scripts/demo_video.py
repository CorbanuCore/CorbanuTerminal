#!/usr/bin/env python3
"""Record a short feature demo of the real Corbanu TUI from a tmux session.

One command drives the candidate in a private tmux server (real keys, real
output), records the attached pane with asciinema, compresses idle gaps, adds a
title card and renders an H.264 MP4. See qa/demos/README.md.

    python3 scripts/demo_video.py record qa/demos/specs/<demo>.toml \
        --bin <candidate> --sprint <SPRINT> [--publish]
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import textwrap
import time
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEMOS_DIR = REPO_ROOT / "qa" / "demos"
RELEASE_TAG = "demos"
REPO_SLUG = "CorbanuCore/CorbanuTerminal"
MAX_SECONDS = 88.0
IDLE_LIMIT = 1.0
TITLE_SECONDS = 5.0
LAST_FRAME_SECONDS = 4.0
TYPE_DELAY = 0.035
KEEPALIVE = (
    "\x1b7\x1b8"  # save/restore cursor: invisible, keeps a deliberate hold on screen
)
STEP_KINDS = ("type", "key", "wait", "pause")
# Generic shapes of real keys; any match blocks rendering and publishing.
SECRET_PATTERNS = [
    re.compile(rb"\bsk-[A-Za-z0-9_\-]{20,}"),
    re.compile(rb"\b[0-9a-f]{32}\.[A-Za-z0-9]{16}\b"),
    re.compile(rb"\bgh[pousr]_[A-Za-z0-9]{30,}"),
    re.compile(rb"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
]


class DemoError(Exception):
    pass


@dataclass
class Step:
    kind: str
    value: object
    timeout: float = 30.0
    repeat: int = 1
    compress: float | None = None
    verify: bool = True


@dataclass
class Spec:
    id: str
    feature: str
    expected: str
    steps: list[Step]
    cols: int = 120
    rows: int = 36
    model: str | None = None
    provider: str | None = None
    args: list[str] = field(default_factory=list)
    config: str = ""
    credentials: dict[str, str] = field(default_factory=dict)
    fixtures: dict[str, str] = field(default_factory=dict)


def load_spec(path: Path) -> Spec:
    data = tomllib.loads(path.read_text())
    for key in ("id", "feature", "expected", "steps"):
        if key not in data:
            raise DemoError(f"{path}: missing `{key}`")
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]*", data["id"]):
        raise DemoError(f"{path}: id must be lowercase letters, digits and dashes")
    steps = []
    for index, raw in enumerate(data["steps"], 1):
        kinds = [k for k in STEP_KINDS if k in raw]
        if len(kinds) != 1:
            raise DemoError(f"{path}: step {index} needs exactly one of {STEP_KINDS}")
        kind = kinds[0]
        steps.append(
            Step(
                kind=kind,
                value=raw[kind],
                timeout=float(raw.get("timeout", 30.0)),
                repeat=int(raw.get("repeat", 1)),
                compress=raw.get("compress"),
                verify=bool(raw.get("verify", True)),
            )
        )
    return Spec(
        id=data["id"],
        feature=data["feature"],
        expected=data["expected"],
        steps=steps,
        cols=int(data.get("cols", 120)),
        rows=int(data.get("rows", 36)),
        model=data.get("model"),
        provider=data.get("provider"),
        args=list(data.get("args", [])),
        config=data.get("config", ""),
        credentials=dict(data.get("credentials", {})),
        fixtures=dict(data.get("fixtures", {})),
    )


def expand(text: str, places: dict[str, str]) -> str:
    for name, value in places.items():
        text = text.replace("{" + name + "}", value)
    return text


# ---------------------------------------------------------------- timeline


def edit_events(
    events: list[list],
    end: float,
    pauses: list[tuple[float, float]],
    compress: list[tuple[float, float, float]],
    idle_limit: float = IDLE_LIMIT,
) -> list[list]:
    """Trim at `end`, keep pause holds, squeeze compress windows, cap other gaps."""
    marks = [[t, "o", KEEPALIVE] for a, b in pauses for t in (a, b) if t <= end]
    kept = sorted(
        [e for e in events if e[0] <= end] + marks + [[end, "o", KEEPALIVE]],
        key=lambda e: e[0],
    )
    out: list[list] = []
    prev_raw = prev_new = 0.0
    for t, kind, data in kept:
        gap = max(0.0, t - prev_raw)
        mid = (prev_raw + t) / 2
        if any(a <= mid <= b for a, b in pauses):
            new_gap = gap
        else:
            window = next(((a, b, d) for a, b, d in compress if a <= mid <= b), None)
            if window and window[1] > window[0]:
                new_gap = min(gap * window[2] / (window[1] - window[0]), idle_limit)
            else:
                new_gap = min(gap, idle_limit)
        prev_raw, prev_new = t, prev_new + new_gap
        out.append([round(prev_new, 6), kind, data])
    return out


def title_events(
    spec: Spec, sha: str, day: str, model: str, seconds: float = TITLE_SECONDS
) -> list[list]:
    width = max(40, spec.cols - 8)
    lines = ["\x1b[1;36mCorbanu Terminal demo\x1b[0m", ""]
    lines += [f"\x1b[1m{part}\x1b[0m" for part in textwrap.wrap(spec.feature, width)]
    lines += ["", "\x1b[33mExpected result\x1b[0m"]
    lines += textwrap.wrap(spec.expected, width)
    lines += [
        "",
        f"\x1b[2mProduct commit\x1b[0m  {sha}",
        f"\x1b[2mRecorded\x1b[0m        {day}",
        f"\x1b[2mModel\x1b[0m           {model}",
        "",
        "\x1b[2mReal keystrokes and real output follow; idle gaps are compressed.\x1b[0m",
    ]
    top = max(1, (spec.rows - len(lines)) // 2)
    body = "\x1b[2J\x1b[H\x1b[?25l" + "".join(
        f"\x1b[{top + i};5H{line}" for i, line in enumerate(lines)
    )
    events = [[0.0, "o", body]]
    events += [[float(s), "o", KEEPALIVE] for s in range(1, int(seconds))]
    events.append(
        [seconds, "o", "\x1bc"]
    )  # full reset before the real recording starts
    return events


def read_cast(path: Path) -> tuple[dict, list[list]]:
    lines = path.read_text(errors="replace").splitlines() if path.exists() else []
    if not lines:
        raise DemoError(f"{path} is empty; asciinema recorded nothing")
    header = json.loads(lines[0])
    if header.get("version") != 2:
        raise DemoError(f"{path}: expected asciicast v2")
    events = [json.loads(line) for line in lines[1:] if line.strip()]
    return header, [e for e in events if e[1] in ("o", "r")]


def write_cast(path: Path, header: dict, events: list[list]) -> None:
    with path.open("w") as handle:
        handle.write(json.dumps(header) + "\n")
        for event in events:
            handle.write(json.dumps(event) + "\n")


# ---------------------------------------------------------------- tmux driver


class Tmux:
    def __init__(self, socket: str):
        self.socket = socket

    def run(self, *args: str, check: bool = True) -> str:
        proc = subprocess.run(
            ["tmux", "-L", self.socket, *args], capture_output=True, text=True
        )
        if check and proc.returncode:
            raise DemoError(f"tmux {args[0]} failed: {proc.stderr.strip()}")
        return proc.stdout

    def capture(self) -> str:
        return self.run("capture-pane", "-p", "-t", "demo")

    def wait(self, pattern: str, timeout: float) -> None:
        """Semantic match on two identical consecutive captures, like the Rust harness."""
        regex = re.compile(pattern, re.S)
        deadline, last = time.monotonic() + timeout, None
        while time.monotonic() < deadline:
            screen = self.capture()
            if regex.search(screen) and screen == last:
                return
            last = screen
            time.sleep(0.2)
        raise DemoError(
            f"timed out after {timeout:.0f}s waiting for /{pattern}/; last screen:\n{(last or '').rstrip()}"
        )


def exit_product(tmux: Tmux, log) -> None:
    """Leave through the product workflow so logs and state are flushed before scanning."""
    tmux.run("send-keys", "-t", "demo", "-l", "/exit")
    time.sleep(0.5)
    tmux.run("send-keys", "-t", "demo", "Enter")
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        if (
            tmux.run(
                "display-message", "-p", "-t", "demo", "#{pane_dead}", check=False
            ).strip()
            == "1"
        ):
            return
        time.sleep(0.2)
    log(
        "WARNING: product did not exit within 20s of /exit; the tmux server will be killed"
    )


def flat(text: str) -> str:
    return re.sub(r"\s+", "", text)


def drive(tmux: Tmux, spec: Spec, places: dict[str, str], t0: float, log) -> dict:
    pauses, compress = [], []
    for number, step in enumerate(spec.steps, 1):
        start = time.monotonic() - t0
        log(f"step {number}: {step.kind} {step.value!r}")
        if step.kind == "type":
            text = expand(str(step.value), places)
            for char in text:
                tmux.run("send-keys", "-t", "demo", "-l", char)
                time.sleep(TYPE_DELAY)
            if step.verify:  # text and Enter are always separate sends
                tail = flat(text)[-24:]
                deadline = time.monotonic() + step.timeout
                while tail not in flat(tmux.capture()):
                    if time.monotonic() > deadline:
                        raise DemoError(f"typed text not visible: {text!r}")
                    time.sleep(0.1)
        elif step.kind == "key":
            for _ in range(step.repeat):
                tmux.run("send-keys", "-t", "demo", str(step.value))
                time.sleep(0.25)
        elif step.kind == "wait":
            tmux.wait(expand(str(step.value), places), step.timeout)
            if step.compress is not None:
                compress.append((start, time.monotonic() - t0, float(step.compress)))
        elif step.kind == "pause":
            time.sleep(float(step.value))
            pauses.append((start, time.monotonic() - t0))
    return {"pauses": pauses, "compress": compress}


# ---------------------------------------------------------------- recording


def git(*args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(REPO_ROOT), *args], capture_output=True, text=True
    ).stdout.strip()


def product_commit() -> str:
    sha = git("rev-parse", "--short=12", "HEAD")
    dirty = git("status", "--porcelain", "--untracked-files=no", "--", "codex-rs")
    return sha + ("-dirty" if dirty else "")


def credential_prefix(spec: Spec, overrides: dict[str, str]) -> str:
    """Shell assignments that resolve each credential at use time, never as literals."""
    sources = {var: f"vault:{label}" for var, label in spec.credentials.items()}
    sources.update(overrides)
    parts = []
    for var, source in sources.items():
        if not re.fullmatch(r"[A-Z_][A-Z0-9_]*", var):
            raise DemoError(f"bad credential variable {var!r}")
        kind, _, ref = source.partition(":")
        if kind == "vault":
            helper = shutil.which("corbanu") or "corbanu"
            parts.append(
                f'{var}="$({shlex.quote(helper)} vault auth-helper {shlex.quote(ref)})"'
            )
        elif kind == "file":
            parts.append(f'{var}="$(cat {shlex.quote(ref)})"')
        else:
            raise DemoError(
                f"credential source must be vault:<label> or file:<path>, got {source!r}"
            )
    return " ".join(parts)


def write_launcher(
    run: Path, spec: Spec, binary: Path, places: dict[str, str], creds: str
) -> Path:
    args = [str(binary), "-C", places["workspace"], "-c", f'log_dir="{places["logs"]}"']
    if spec.model:
        args += ["-m", spec.model]
    if spec.provider:
        args += ["-c", f'model_provider="{spec.provider}"']
    args += [expand(a, places) for a in spec.args]
    env = {
        "HOME": places["userhome"],
        "CODEX_HOME": places["home"],
        "CORBANU_HOME": places["home"],
        "PFTERMINAL_HOME": places["home"],
        "CORBANU_TEST_NO_NATIVE_KEYRING": "1",
        "RUST_LOG": "trace",
        "TERM": "xterm-256color",
    }
    # Credentials resolve before HOME moves to the disposable profile.
    script = f"""#!/bin/bash
# Generated by scripts/demo_video.py. Holds until the recorder is attached.
while [ ! -e {shlex.quote(str(run / "go"))} ]; do sleep 0.05; done
for v in $(compgen -e); do
  case "$v" in *API_KEY*|*TOKEN*|*SECRET*|*PASSWORD*|*CREDENTIAL*|*_AUTH*) unset "$v";; esac
done
cd {shlex.quote(places["workspace"])}
{creds + " " if creds else ""}exec env {" ".join(f"{k}={shlex.quote(v)}" for k, v in env.items())} \\
  {" ".join(shlex.quote(a) for a in args)}
"""
    path = run / "launch.sh"
    path.write_text(script)
    return path


def prepare_run(spec: Spec, out: Path) -> tuple[Path, dict[str, str]]:
    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    run = (out / f"{spec.id}-{stamp}").resolve()
    names = ("home", "workspace", "outside", "logs", "userhome")
    places = {name: str(run / name) for name in names}
    places["run"] = str(run)
    for name in names:
        Path(places[name]).mkdir(parents=True)
    workspace = Path(places["workspace"])
    for rel, content in spec.fixtures.items():
        target = (workspace / rel).resolve()
        if workspace not in target.parents:
            raise DemoError(f"fixture escapes workspace: {rel}")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(expand(content, places))
    subprocess.run(["git", "init", "-q", str(workspace)], check=True)
    config = expand(spec.config, places)
    if "[projects." not in config:
        config += f'\n[projects."{places["workspace"]}"]\ntrust_level = "trusted"\n'
    (Path(places["home"]) / "config.toml").write_text(config)
    return run, places


def scan_files(
    published: list[Path], private: list[Path], env_names: list[str]
) -> dict:
    """Scan without printing matches.

    Published files (the cast) must hold no credential value and no key-shaped
    string. Private run files (logs, profile) are redacted in place, same length,
    and their paths are reported as a product finding.
    """
    needles = [os.environ[n].encode() for n in env_names if os.environ.get(n)]
    hits = {
        "credentials_checked": len(needles),
        "published_literal": 0,
        "published_pattern": 0,
        "redacted_private_files": [],
    }
    for path in published:
        data = path.read_bytes() if path.is_file() else b""
        hits["published_literal"] += sum(data.count(n) for n in needles)
        hits["published_pattern"] += sum(len(p.findall(data)) for p in SECRET_PATTERNS)
    for path in private:
        if not path.is_file() or path.is_symlink():
            continue
        data = path.read_bytes()
        if any(n in data for n in needles):
            for needle in needles:
                data = data.replace(needle, b"*" * len(needle))
            path.write_bytes(data)
            hits["redacted_private_files"].append(str(path))
    return hits


def secret_scan(creds: str, published: list[Path], private: list[Path]) -> dict:
    names = re.findall(r"([A-Z_][A-Z0-9_]*)=\"\$\(", creds)
    cmd = " ".join(
        [creds, shlex.quote(sys.executable), shlex.quote(__file__), "_scan"]
        + [f"--env {n}" for n in names]
    )
    listing = "\n".join([f"P\t{p}" for p in published] + [f"R\t{p}" for p in private])
    proc = subprocess.run(
        ["bash", "-c", cmd], input=listing, capture_output=True, text=True
    )
    if proc.returncode:
        raise DemoError(f"secret scan failed to run: {proc.stderr.strip()[-300:]}")
    return json.loads(proc.stdout)


def render(cast: Path, mp4: Path, speed: float) -> float:
    gif = cast.with_suffix(".gif")
    subprocess.run(
        [
            "agg",
            "-q",
            "--idle-time-limit",
            "1000",
            "--last-frame-duration",
            str(LAST_FRAME_SECONDS),
            "--font-size",
            "16",
            "--speed",
            f"{speed:.3f}",
            "--no-loop",
            str(cast),
            str(gif),
        ],
        check=True,
    )
    subprocess.run(
        [
            "ffmpeg",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(gif),
            "-movflags",
            "+faststart",
            "-vf",
            "fps=30,scale=trunc(iw/2)*2:trunc(ih/2)*2",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
            "-preset",
            "slow",
            "-crf",
            "18",
            str(mp4),
        ],
        check=True,
    )
    gif.unlink()
    return duration(mp4)


def duration(path: Path) -> float:
    out = subprocess.run(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            str(path),
        ],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return float(out.strip())


def extract_frames(mp4: Path, seconds: float, folder: Path) -> list[Path]:
    folder.mkdir(exist_ok=True)
    frames = []
    for label, at in (
        ("title", 2.0),
        ("middle", seconds / 2),
        ("final", max(0.0, seconds - 0.5)),
    ):
        frame = folder / f"{label}.png"
        subprocess.run(
            [
                "ffmpeg",
                "-loglevel",
                "error",
                "-y",
                "-ss",
                f"{at:.2f}",
                "-i",
                str(mp4),
                "-frames:v",
                "1",
                str(frame),
            ],
            check=True,
        )
        frames.append(frame)
    return frames


def stop_recording(tmux: Tmux, recorder: subprocess.Popen | None) -> None:
    tmux.run("kill-server", check=False)  # private -L socket only
    if recorder:
        try:
            recorder.wait(timeout=15)
        except subprocess.TimeoutExpired:
            recorder.terminate()
            recorder.wait(timeout=5)


def private_files(places: dict[str, str]) -> list[Path]:
    roots = ("logs", "home", "userhome", "workspace", "outside")
    return [p for root in roots for p in sorted(Path(places[root]).rglob("*"))]


def record(args: argparse.Namespace) -> Path:
    for tool in ("tmux", "asciinema", "agg", "ffmpeg", "ffprobe"):
        if not shutil.which(tool):
            raise DemoError(f"{tool} not found; see qa/demos/README.md for setup")
    spec = load_spec(Path(args.spec))
    binary = Path(args.bin).resolve()
    if not os.access(binary, os.X_OK):
        raise DemoError(f"candidate binary not executable: {binary}")
    overrides = dict(item.split("=", 1) for item in args.credential)
    creds = credential_prefix(spec, overrides)
    sha, day = product_commit(), dt.date.today().isoformat()
    run, places = prepare_run(spec, Path(args.out) / args.sprint)

    def log(message: str) -> None:
        line = f"[{time.strftime('%H:%M:%S')}] {message}"
        print(line, flush=True)
        with (run / "driver.log").open("a") as handle:
            handle.write(line + "\n")

    version = subprocess.run(
        [str(binary), "--version"], capture_output=True, text=True
    ).stdout.strip()
    launcher = write_launcher(run, spec, binary, places, creds)
    tmux = Tmux(f"demo-{spec.id}-{os.getpid()}")
    raw_cast = run / "raw.cast"
    recorder = None
    try:
        tmux.run(
            "-f",
            "/dev/null",
            "new-session",
            "-d",
            "-s",
            "demo",
            "-x",
            str(spec.cols),
            "-y",
            str(spec.rows),
            f"bash {shlex.quote(str(launcher))}",
        )
        for option in (
            ("status", "off"),
            ("remain-on-exit", "on"),
            ("window-size", "manual"),
            ("default-terminal", "xterm-256color"),
        ):
            tmux.run("set-option", "-g", *option)
        tmux.run("set-option", "-s", "escape-time", "0")
        tmux.run(
            "resize-window", "-t", "demo", "-x", str(spec.cols), "-y", str(spec.rows)
        )
        t0 = time.monotonic()
        recorder = subprocess.Popen(
            [
                "asciinema",
                "rec",
                "--headless",
                "-q",
                "--overwrite",
                "-f",
                "asciicast-v2",
                "--window-size",
                f"{spec.cols}x{spec.rows}",
                "-c",
                f"tmux -L {tmux.socket} attach -t demo",
                str(raw_cast),
            ],
            env=dict(os.environ, TERM="xterm-256color"),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        deadline = time.monotonic() + 10
        while not tmux.run("list-clients", check=False).strip():
            if time.monotonic() > deadline or recorder.poll() is not None:
                raise DemoError("asciinema did not attach to the tmux session")
            time.sleep(0.05)
        (run / "go").touch()
        timeline = drive(tmux, spec, places, t0, log)
        end = time.monotonic() - t0
        (run / "final-screen.txt").write_text(tmux.capture())
        log("steps complete; exiting through /exit")
        exit_product(tmux, log)
    except BaseException:
        (run / "failure-screen.txt").write_text(
            tmux.run("capture-pane", "-p", "-t", "demo", check=False)
        )
        stop_recording(tmux, recorder)
        secret_scan(creds, [], private_files(places))  # redact private logs, then fail
        raise
    stop_recording(tmux, recorder)
    scan = secret_scan(creds, [raw_cast], private_files(places))
    manifest = {
        "id": spec.id,
        "sprint": args.sprint,
        "feature": spec.feature,
        "expected": spec.expected,
        "commit": sha,
        "date": day,
        "version": version,
        "model": spec.model or "configured default",
        "provider": spec.provider,
        "spec": os.path.relpath(Path(args.spec).resolve(), REPO_ROOT),
        "secret_scan": scan,
        "run_dir": str(run),
    }
    if scan["redacted_private_files"]:
        log(
            f"PRODUCT FINDING: credential value found and redacted in {scan['redacted_private_files']}"
        )
    if scan["published_literal"] or scan["published_pattern"]:
        manifest["blocked"] = "secret scan matched; not rendered"
        (run / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        raise DemoError(
            f"secret scan matched {scan}; recording kept private in {run} and NOT rendered"
        )

    header, events = read_cast(raw_cast)
    edited = edit_events(events, end, timeline["pauses"], timeline["compress"])
    model = (
        f"{spec.model} ({spec.provider})"
        if spec.provider
        else (spec.model or "configured default")
    )
    title = title_events(spec, sha, day, model)
    shift = title[-1][0]
    combined = title + [[round(t + shift, 6), k, d] for t, k, d in edited]
    header = {
        "version": 2,
        "width": spec.cols,
        "height": spec.rows,
        "timestamp": header.get("timestamp"),
        "title": f"{spec.feature} ({sha})",
        "env": {"TERM": "xterm-256color"},
    }
    base = f"{args.sprint}-{spec.id}-{sha.split('-')[0]}-{day}".lower()
    cast, mp4 = run / f"{base}.cast", run / f"{base}.mp4"
    write_cast(cast, header, combined)
    speed = max(1.0, (combined[-1][0] + LAST_FRAME_SECONDS) / MAX_SECONDS)
    seconds = render(cast, mp4, speed)
    if seconds > 90:
        raise DemoError(
            f"{mp4} is {seconds:.1f}s; add `compress` to slow waits or split the demo"
        )
    frames = extract_frames(mp4, seconds, run / "frames")
    manifest.update(
        {
            "cast": str(cast),
            "raw_cast": str(raw_cast),
            "mp4": str(mp4),
            "seconds": round(seconds, 1),
            "speed": round(speed, 3),
            "frames": [str(f) for f in frames],
        }
    )
    (run / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    log(f"rendered {mp4} ({seconds:.1f}s); frames in {run / 'frames'}")
    if args.publish:
        publish_run(run)
    return run


# ---------------------------------------------------------------- publishing


INDEX_HEADER = """# {sprint} demo videos

Recorded with `scripts/demo_video.py`; see [the demo SOP](../README.md).
Videos are assets on the [`demos` prerelease](https://github.com/{repo}/releases/tag/demos).

| Date | Demo | Feature | Commit | Model | Length | Video |
| --- | --- | --- | --- | --- | --- | --- |
"""


def index_row(manifest: dict, url: str, cast_url: str) -> str:
    return (
        f"| {manifest['date']} | `{manifest['id']}` | {manifest['feature']} | `{manifest['commit']}` | "
        f"{manifest['model']} | {manifest['seconds']:.0f}s | [mp4]({url}) · [cast]({cast_url}) |"
    )


def update_index(manifest: dict, url: str, cast_url: str, index_dir: Path) -> Path:
    index_dir.mkdir(parents=True, exist_ok=True)
    path = index_dir / f"{manifest['sprint']}.md"
    text = (
        path.read_text()
        if path.exists()
        else INDEX_HEADER.format(sprint=manifest["sprint"], repo=REPO_SLUG)
    )
    lines = [line for line in text.splitlines() if f"[mp4]({url})" not in line]
    lines.append(index_row(manifest, url, cast_url))
    path.write_text("\n".join(lines) + "\n")
    return path


def publish_run(run: Path) -> None:
    manifest = json.loads((run / "manifest.json").read_text())
    if manifest.get("blocked") or "mp4" not in manifest:
        raise DemoError(f"{run} is not publishable")
    exists = subprocess.run(
        ["gh", "release", "view", RELEASE_TAG, "-R", REPO_SLUG], capture_output=True
    )
    if exists.returncode:
        subprocess.run(
            [
                "gh",
                "release",
                "create",
                RELEASE_TAG,
                "-R",
                REPO_SLUG,
                "--prerelease",
                "--target",
                "main",
                "--title",
                "Demo videos",
                "--notes",
                "Feature demo videos recorded from the real TUI. Index: qa/demos/index/. SOP: qa/demos/README.md.",
            ],
            check=True,
        )
    subprocess.run(
        [
            "gh",
            "release",
            "upload",
            RELEASE_TAG,
            "-R",
            REPO_SLUG,
            "--clobber",
            manifest["mp4"],
            manifest["cast"],
        ],
        check=True,
    )
    base = f"https://github.com/{REPO_SLUG}/releases/download/{RELEASE_TAG}/"
    url, cast_url = (
        base + Path(manifest["mp4"]).name,
        base + Path(manifest["cast"]).name,
    )
    manifest["url"], manifest["cast_url"] = url, cast_url
    (run / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    index = update_index(manifest, url, cast_url, DEMOS_DIR / "index")
    print(f"published {url}\nindex {index}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="command", required=True)
    rec = sub.add_parser(
        "record", help="record, render and optionally publish one demo"
    )
    rec.add_argument("spec")
    rec.add_argument(
        "--bin",
        required=True,
        help="candidate binary, e.g. $CARGO_TARGET_DIR/debug/corbanu",
    )
    rec.add_argument(
        "--sprint",
        required=True,
        help="sprint id used in names and the index, e.g. PF-24-S02",
    )
    rec.add_argument(
        "--out",
        default=str(DEMOS_DIR / "out"),
        help="run directory root (never committed)",
    )
    rec.add_argument(
        "--credential", action="append", default=[], metavar="VAR=vault:LABEL|file:PATH"
    )
    rec.add_argument(
        "--publish",
        action="store_true",
        help="upload to the `demos` prerelease and update the index",
    )
    pub = sub.add_parser("publish", help="publish an already recorded run directory")
    pub.add_argument("run_dir")
    scan = sub.add_parser("_scan", help=argparse.SUPPRESS)
    scan.add_argument("--env", action="append", default=[])
    args = parser.parse_args(argv)
    try:
        if args.command == "record":
            record(args)
        elif args.command == "publish":
            publish_run(Path(args.run_dir))
        else:
            rows = [
                line.split("\t", 1)
                for line in sys.stdin.read().splitlines()
                if "\t" in line
            ]
            published = [Path(p) for kind, p in rows if kind == "P"]
            private = [Path(p) for kind, p in rows if kind == "R"]
            print(json.dumps(scan_files(published, private, args.env)))
    except (DemoError, subprocess.CalledProcessError) as err:
        print(f"demo_video: {err}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

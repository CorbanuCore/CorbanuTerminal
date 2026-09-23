"""Docker isolation and OpenRouter relay lifecycle for leak-isolated campaigns.

Each contestant run gets one container that can see exactly two host paths:
its fresh candidate workspace (``/workspace``) and an empty agent home
(``/home/bench``). The container sits on an internal Docker network with no
internet route; ``openrouter.ai`` resolves to the campaign relay (see
``relay.py``), which holds the only real provider key.
"""

from __future__ import annotations

import hashlib
import json
import os
import secrets
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ISOLATION_DIR = Path(__file__).resolve().parent
RELAY_SOURCE = ISOLATION_DIR / "relay.py"
CONTAINER_HOME = "/home/bench"
CONTAINER_WORKSPACE = "/workspace"
CONTAINER_CA = "/bench-ca/ca.pem"
# Toolsets that fetch network content. They are disabled identically for every
# harness; the network boundary would block them anyway.
HERMES_DISABLED_TOOLSETS = ["web", "search", "browser", "x_search"]


def _docker(*args: str, check: bool = True, env: dict[str, str] | None = None) -> str:
    completed = subprocess.run(
        ["docker", *args],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )
    if check and completed.returncode != 0:
        raise RuntimeError(f"docker {args[0]} failed: {completed.stderr.strip()[:500]}")
    return completed.stdout.strip()


def _openssl(*args: str) -> None:
    subprocess.run(["openssl", *args], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


@dataclass(frozen=True)
class IsolationSpec:
    images: dict[str, str]
    relay_image: str = "python:3.12-slim-bookworm"
    upstream_key_env: str = "OPENROUTER_API_KEY"
    cpus: str = "4"
    memory: str = "8g"
    pids_limit: int = 2048

    @classmethod
    def from_config(cls, raw: dict[str, Any]) -> "IsolationSpec":
        if raw.get("mode") != "docker":
            raise ValueError("isolation.mode must be 'docker'")
        images = raw.get("images")
        if not isinstance(images, dict) or not images:
            raise ValueError("isolation.images must map each harness kind to its own image")
        return cls(
            images={str(kind): str(image) for kind, image in images.items()},
            relay_image=str(raw.get("relay_image") or cls.relay_image),
            upstream_key_env=str(raw.get("upstream_key_env") or cls.upstream_key_env),
            cpus=str(raw.get("cpus") or cls.cpus),
            memory=str(raw.get("memory") or cls.memory),
            pids_limit=int(raw.get("pids_limit") or cls.pids_limit),
        )


class Campaign:
    """Owns the internal network, TLS material and relay for one run root."""

    def __init__(self, spec: IsolationSpec, run_root: Path) -> None:
        self.spec = spec
        self.run_root = run_root
        self.relay_root = run_root / "relay"
        self.records = self.relay_root / "records"
        self.tls = self.relay_root / "tls"
        suffix = hashlib.sha256(str(run_root).encode()).hexdigest()[:10]
        self.network = f"cbench-{suffix}"
        self.relay_name = f"cbench-{suffix}-relay"
        self.relay_ip: str | None = None

    # -- lifecycle ---------------------------------------------------------
    def start(self) -> dict[str, Any]:
        key = os.environ.get(self.spec.upstream_key_env, "").strip()
        if not key:
            raise RuntimeError(f"{self.spec.upstream_key_env} is required for the relay")
        for image in self.spec.images.values():
            _docker("image", "inspect", image)
        (self.records / "registrations").mkdir(parents=True, exist_ok=True)
        self._issue_certificate()
        _docker("network", "create", "--internal", self.network)
        env = {**os.environ, "OPENROUTER_API_KEY": key}
        _docker(
            "run", "-d", "--name", self.relay_name,
            "--user", f"{os.getuid()}:{os.getgid()}",
            "--sysctl", "net.ipv4.ip_unprivileged_port_start=443",
            "--env", "OPENROUTER_API_KEY",
            "--mount", f"type=bind,src={RELAY_SOURCE},dst=/relay.py,readonly",
            "--mount", f"type=bind,src={self.tls / 'relay.pem'},dst=/tls/relay.pem,readonly",
            "--mount", f"type=bind,src={self.tls / 'relay.key'},dst=/tls/relay.key,readonly",
            "--mount", f"type=bind,src={self.records},dst=/records",
            self.spec.relay_image,
            "python3", "/relay.py", "--records", "/records",
            "--cert", "/tls/relay.pem", "--key", "/tls/relay.key", "--port", "443",
            env=env,
        )
        _docker("network", "connect", self.network, self.relay_name)
        self.relay_ip = _docker(
            "inspect", "-f", f'{{{{(index .NetworkSettings.Networks "{self.network}").IPAddress}}}}',
            self.relay_name,
        )
        time.sleep(1.0)
        if _docker("inspect", "-f", "{{.State.Running}}", self.relay_name) != "true":
            raise RuntimeError("relay container exited: " + _docker("logs", self.relay_name, check=False))
        return {
            "network": self.network,
            "relay": self.relay_name,
            "relay_ip": self.relay_ip,
            "images": {
                kind: {"tag": image, "id": _docker("image", "inspect", "-f", "{{.Id}}", image)}
                for kind, image in self.spec.images.items()
            },
            "relay_source_sha256": hashlib.sha256(RELAY_SOURCE.read_bytes()).hexdigest(),
        }

    def stop(self) -> None:
        _docker("rm", "-f", self.relay_name, check=False)
        _docker("network", "rm", self.network, check=False)

    def _issue_certificate(self) -> None:
        self.tls.mkdir(parents=True, exist_ok=True)
        ca_key, ca = self.tls / "ca.key", self.tls / "ca.pem"
        key, csr, cert = self.tls / "relay.key", self.tls / "relay.csr", self.tls / "relay.pem"
        ext = self.tls / "relay.ext"
        _openssl("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", str(ca_key), "-out", str(ca),
                 "-days", "14", "-subj", "/CN=Corbanu Benchmark Relay CA",
                 "-addext", "basicConstraints=critical,CA:TRUE",
                 "-addext", "keyUsage=critical,keyCertSign,cRLSign")
        _openssl("req", "-newkey", "rsa:2048", "-nodes", "-keyout", str(key), "-out", str(csr),
                 "-subj", "/CN=openrouter.ai")
        ext.write_text(
            "subjectAltName=DNS:openrouter.ai\nbasicConstraints=CA:FALSE\n"
            "extendedKeyUsage=serverAuth\nkeyUsage=digitalSignature,keyEncipherment\n",
            encoding="utf-8",
        )
        _openssl("x509", "-req", "-in", str(csr), "-CA", str(ca), "-CAkey", str(ca_key),
                 "-CAcreateserial", "-out", str(cert), "-days", "14", "-extfile", str(ext))
        # The CA key is only needed to sign this one leaf; nothing may mint more.
        ca_key.unlink()
        csr.unlink()
        key.chmod(0o600)

    # -- per run -----------------------------------------------------------
    def register(self, run_id: str, model: str, timeout_seconds: int) -> tuple[str, Path]:
        token = secrets.token_urlsafe(32)
        path = self.records / "registrations" / f"{hashlib.sha256(token.encode()).hexdigest()}.json"
        path.write_text(
            json.dumps({"run_id": run_id, "model": model, "expires_at": time.time() + timeout_seconds + 300}),
            encoding="utf-8",
        )
        return token, path

    def container_argv(
        self,
        *,
        name: str,
        kind: str,
        workspace: Path,
        home: Path,
        env_file: Path,
        interactive: bool,
        agent_argv: list[str],
    ) -> list[str]:
        if not self.relay_ip:
            raise RuntimeError("campaign relay is not running")
        return [
            "docker", "run", "--rm", "--init", "--name", name,
            "--network", self.network, "--add-host", f"openrouter.ai:{self.relay_ip}",
            "--user", f"{os.getuid()}:{os.getgid()}",
            "--cpus", self.spec.cpus, "--memory", self.spec.memory,
            "--pids-limit", str(self.spec.pids_limit),
            "--mount", f"type=bind,src={workspace},dst={CONTAINER_WORKSPACE}",
            "--mount", f"type=bind,src={home},dst={CONTAINER_HOME}",
            "--mount", f"type=bind,src={self.tls / 'ca.pem'},dst={CONTAINER_CA},readonly",
            "--env-file", str(env_file),
            "-w", CONTAINER_WORKSPACE,
            *(["-i"] if interactive else []),
            self.spec.images[kind],
            *agent_argv,
        ]

    def run_records(self, run_id: str) -> Path:
        return self.records / "runs" / run_id


def kill_container(name: str) -> None:
    _docker("kill", name, check=False)
    _docker("rm", "-f", name, check=False)


def prepare_agent(
    kind: str,
    model: str,
    prompt: str,
    home: Path,
    token: str,
    extra_args: tuple[str, ...] = (),
) -> tuple[list[str], dict[str, str], str | None]:
    """Write the harness's fresh home and return (argv, env, stdin payload).

    Every harness uses its native OpenRouter provider and its own defaults;
    only network-content tools are disabled.
    """

    home.mkdir(parents=True, exist_ok=True)
    env = {
        "OPENROUTER_API_KEY": token,
        "SSL_CERT_FILE": CONTAINER_CA,
        "REQUESTS_CA_BUNDLE": CONTAINER_CA,
        "NODE_EXTRA_CA_CERTS": CONTAINER_CA,
        "NO_COLOR": "1",
    }
    if kind == "corbanu":
        (home / ".corbanu").mkdir(exist_ok=True)
        env["CODEX_HOME"] = f"{CONTAINER_HOME}/.corbanu"
        argv = [
            "corbanu", "exec", "--json", "--skip-git-repo-check",
            "--dangerously-bypass-approvals-and-sandbox", "-C", CONTAINER_WORKSPACE,
            "-c", 'model_provider="openrouter"', "-m", model, *extra_args, "-",
        ]
        return argv, env, prompt
    if kind == "hermes":
        hermes_home = home / ".hermes"
        hermes_home.mkdir(exist_ok=True)
        (hermes_home / "config.yaml").write_text(
            "agent:\n  disabled_toolsets: [" + ", ".join(HERMES_DISABLED_TOOLSETS) + "]\n",
            encoding="utf-8",
        )
        env["HERMES_HOME"] = f"{CONTAINER_HOME}/.hermes"
        # Hermes's documented runtime working-directory carrier; without it the
        # file tools anchor at $HOME inside a container.
        env["TERMINAL_CWD"] = CONTAINER_WORKSPACE
        argv = [
            "hermes", "--provider", "openrouter", "-m", model, "--yolo", "--accept-hooks",
            "--usage-file", f"{CONTAINER_HOME}/hermes-usage.json", *extra_args, "-z", prompt,
        ]
        return argv, env, None
    if kind == "kilo":
        config_dir = home / ".config" / "kilo"
        config_dir.mkdir(parents=True, exist_ok=True)
        (config_dir / "kilo.json").write_text(
            json.dumps(
                {
                    "$schema": "https://app.kilo.ai/config.json",
                    "autoupdate": False,
                    "share": "disabled",
                    "tools": {"webfetch": False, "websearch": False},
                },
                indent=2,
            ),
            encoding="utf-8",
        )
        # Kilo quotes and backslash-escapes a multi-line argv message, so the
        # model would see a corrupted prompt. Piped stdin arrives verbatim.
        argv = [
            "kilo", "run", "--model", f"openrouter/{model}", "--dir", CONTAINER_WORKSPACE,
            "--format", "json", "--auto", "--title", "benchmark", *extra_args,
        ]
        return argv, env, prompt
    raise ValueError(f"docker isolation does not support agent kind {kind!r}")


def write_env_file(path: Path, env: dict[str, str]) -> None:
    lines = []
    for key, value in env.items():
        if "\n" in value:
            raise ValueError(f"environment value for {key} must be a single line")
        lines.append(f"{key}={value}")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    path.chmod(0o600)


def summarize_run_records(records: Path, expected_model: str) -> dict[str, Any]:
    """Route, spend and boundary evidence captured by the relay for one run."""

    usage_rows = _jsonl(records / "usage.jsonl")
    events = _jsonl(records / "events.jsonl")
    rejected = [event for event in events if event.get("decision") == "rejected"]
    served = sorted({model for row in usage_rows for model in row.get("observed_models") or []})
    costs = [
        float(row["usage"]["cost"])
        for row in usage_rows
        if isinstance(row.get("usage"), dict) and isinstance(row["usage"].get("cost"), (int, float))
    ]
    ok_rows = [row for row in usage_rows if row.get("status") == 200]
    missing_cost = [
        row.get("request_id")
        for row in ok_rows
        if not isinstance((row.get("usage") or {}).get("cost"), (int, float))
    ]
    return {
        "inference_requests": len(usage_rows),
        "successful_inference_requests": len(ok_rows),
        "rejected_requests": len(rejected),
        "rejection_reasons": sorted({str(event.get("reason")) for event in rejected}),
        "upstream_errors": sum(1 for event in events if event.get("decision") == "upstream_error"),
        "models_served": served,
        "route_verified": bool(ok_rows) and all(
            model == expected_model or model.startswith(expected_model + "-")
            for model in served
        ),
        "relay_cost_usd": round(sum(costs), 8) if costs else None,
        "requests_missing_cost": missing_cost,
        "generation_ids": [gid for row in usage_rows for gid in row.get("generation_ids") or []],
        "usage_totals": _usage_totals(usage_rows),
    }


def _usage_totals(rows: list[dict[str, Any]]) -> dict[str, int]:
    totals: dict[str, int] = {}
    for row in rows:
        usage = row.get("usage") or {}
        for key in ("prompt_tokens", "completion_tokens", "input_tokens", "output_tokens"):
            value = usage.get(key)
            if isinstance(value, int):
                totals[key] = totals.get(key, 0) + value
        details = usage.get("prompt_tokens_details") or {}
        if isinstance(details.get("cached_tokens"), int):
            totals["cached_tokens"] = totals.get("cached_tokens", 0) + details["cached_tokens"]
    return totals


def _jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.is_file():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict):
            rows.append(value)
    return rows


def docker_available() -> bool:
    return shutil.which("docker") is not None

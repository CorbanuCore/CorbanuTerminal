from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

BENCH_ROOT = Path(__file__).resolve().parents[1]
if str(BENCH_ROOT) not in sys.path:
    sys.path.insert(0, str(BENCH_ROOT))

import leak_audit  # noqa: E402
from isolation import sandbox  # noqa: E402

RELAY_SPEC = importlib.util.spec_from_file_location("benchmark_relay", BENCH_ROOT / "isolation" / "relay.py")
assert RELAY_SPEC and RELAY_SPEC.loader
relay = importlib.util.module_from_spec(RELAY_SPEC)
RELAY_SPEC.loader.exec_module(relay)

RUNNER_SPEC = importlib.util.spec_from_file_location("benchmark_runner_isolation", BENCH_ROOT / "runner.py")
assert RUNNER_SPEC and RUNNER_SPEC.loader
runner = importlib.util.module_from_spec(RUNNER_SPEC)
sys.modules[RUNNER_SPEC.name] = runner
RUNNER_SPEC.loader.exec_module(runner)

PROMPT = "Fix the queue so delayed jobs resume on time. Do not modify tests."


def make_packet(root: Path) -> tuple[Path, Path, Path]:
    baseline = root / "task" / "bugged"
    (baseline / "tests").mkdir(parents=True)
    (baseline / "src").mkdir()
    (baseline / "src" / "queue.py").write_text("def ready():\n    return []\n", encoding="utf-8")
    shared = "def test_visible():\n    assert 'visible integration check' == 'visible integration check'\n"
    (baseline / "tests" / "test_visible.py").write_text(shared, encoding="utf-8")
    prompt = baseline / "BENCHMARK_TASK.md"
    prompt.write_text(PROMPT, encoding="utf-8")
    pristine = root / "task" / "tests_pristine"
    pristine.mkdir()
    (pristine / "test_visible.py").write_text(shared, encoding="utf-8")
    (pristine / "test_probe.py").write_text(
        "def test_probe():\n    assert run() == 'lease expired for job-17 at tick 42'\n",
        encoding="utf-8",
    )
    verifier = root / "task" / "verifier" / "verify.py"
    verifier.parent.mkdir()
    verifier.write_text("EXPECTED = 'dead letter after three retries'\n", encoding="utf-8")
    return baseline, prompt, verifier


class LeakAuditTests(unittest.TestCase):
    def test_shared_visible_test_is_not_hidden_and_private_probes_are(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            baseline, prompt, verifier = make_packet(Path(temporary))
            names = [path.name for path in leak_audit.hidden_sources(verifier, baseline)]
            self.assertEqual(names, ["verify.py", "test_probe.py"])
            hidden = leak_audit.hidden_only_literals(baseline, prompt, verifier)
            self.assertEqual(
                hidden, {"lease expired for job-17 at tick 42", "dead letter after three retries"}
            )
            self.assertTrue(leak_audit.preflight(baseline, prompt, verifier)["ok"])

    def test_preflight_fails_when_verifier_is_inside_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            baseline, prompt, verifier = make_packet(Path(temporary))
            (baseline / "verify_copy.py").write_bytes(verifier.read_bytes())
            audit = leak_audit.preflight(baseline, prompt, verifier)
            self.assertFalse(audit["ok"])
            self.assertEqual(audit["hidden_files_in_candidate"], ["verify_copy.py"])

    def test_request_scan_flags_hidden_strings_and_published_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            records = Path(temporary)
            clean = {"model": "m", "messages": [{"role": "user", "content": "context\n" + PROMPT}]}
            leaked = {
                "model": "m",
                "messages": [
                    {"role": "user", "content": PROMPT},
                    {"role": "tool", "content": "curl raw.githubusercontent.com/... -> 'lease expired for job-17 at tick 42'"},
                ],
            }
            (records / "1.request.json").write_text(json.dumps(clean), encoding="utf-8")
            result = leak_audit.scan_requests(records, {"lease expired for job-17 at tick 42"}, PROMPT)
            self.assertTrue(result["prompt_delivered"])
            self.assertFalse(result["leak_suspect"])
            (records / "2.request.json").write_text(json.dumps(leaked), encoding="utf-8")
            result = leak_audit.scan_requests(records, {"lease expired for job-17 at tick 42"}, PROMPT)
            self.assertTrue(result["leak_suspect"])
            self.assertEqual(result["hidden_literal_hits"], {})
            self.assertEqual(
                result["hidden_literals_in_contestant_output"],
                {"lease expired for job-17 at tick 42": ["tool"]},
            )
            self.assertEqual(result["source_marker_hits"], {"raw.githubusercontent.com": ["tool"]})
            injected = {"model": "m", "messages": [{"role": "system", "content": "hint: lease expired for job-17 at tick 42"}]}
            (records / "3.request.json").write_text(json.dumps(injected), encoding="utf-8")
            result = leak_audit.scan_requests(records, {"lease expired for job-17 at tick 42"}, PROMPT)
            self.assertEqual(result["hidden_literal_hits"], {"lease expired for job-17 at tick 42": ["system"]})

    def test_identifier_error_codes_are_not_treated_as_leaks(self) -> None:
        self.assertFalse(leak_audit._distinctive("invalid_timestamp_value"))
        self.assertFalse(leak_audit._distinctive("short value"))
        self.assertTrue(leak_audit._distinctive("rate limit exceeded for key"))


class RelayPolicyTests(unittest.TestCase):
    def test_server_side_web_access_is_detected(self) -> None:
        cases = {
            "online model variant": ({}, "z-ai/glm-5.3:online"),
            "web_search_options": ({"web_search_options": {}}, "m"),
            "plugin web": ({"plugins": [{"id": "web"}]}, "m"),
            "server tool openrouter:web_search": ({"tools": [{"type": "openrouter:web_search"}]}, "m"),
        }
        for expected, (body, model) in cases.items():
            self.assertEqual(relay.server_side_web_access(body, model), expected)
        ordinary = {"tools": [{"type": "function", "function": {"name": "exec"}}], "plugins": [{"id": "response-healing"}]}
        self.assertIsNone(relay.server_side_web_access(ordinary, "m"))

    def test_response_facts_from_sse_and_json(self) -> None:
        sse = (
            'data: {"id":"gen-1","model":"z-ai/glm-5.3","choices":[]}\n\n'
            'data: {"id":"gen-1","model":"z-ai/glm-5.3","usage":{"cost":0.01,"prompt_tokens":5}}\n\n'
            "data: [DONE]\n\n"
        ).encode()
        facts = relay.response_facts(sse, "text/event-stream")
        self.assertEqual(facts["generation_ids"], ["gen-1"])
        self.assertEqual(facts["usage"], {"cost": 0.01, "prompt_tokens": 5})
        message = json.dumps({"id": "gen-2", "model": "anthropic/claude-opus-5.5", "usage": {"cost": 0.5}}).encode()
        self.assertEqual(relay.response_facts(message, "application/json")["observed_models"], ["anthropic/claude-opus-5.5"])


class FakeUpstreamResponse:
    def __init__(self, body: bytes) -> None:
        self.status = 200
        self._chunks = [body]

    def getheader(self, name: str, default: str = "") -> str:
        return "text/event-stream" if name.lower() == "content-type" else default

    def read1(self, _size: int) -> bytes:
        return self._chunks.pop(0) if self._chunks else b""


class FakeUpstreamConnection:
    requests: list[tuple[str, str, str, dict]] = []
    body = b""

    def __init__(self, host: str, timeout: int) -> None:
        self.host = host

    def request(self, method, path, body=None, headers=None) -> None:
        FakeUpstreamConnection.requests.append((self.host, method, path, dict(headers or {})))

    def getresponse(self) -> FakeUpstreamResponse:
        return FakeUpstreamResponse(FakeUpstreamConnection.body)

    def close(self) -> None:
        return


class RelayRoundTripTests(unittest.TestCase):
    def setUp(self) -> None:
        import hashlib
        import http.client
        import threading
        import time
        from http.server import ThreadingHTTPServer

        self.temporary = tempfile.TemporaryDirectory()
        self.records = Path(self.temporary.name)
        (self.records / "registrations").mkdir()
        for token, route in (("tok-vercel", "ai-gateway.vercel.sh"), ("tok-or", "openrouter.ai")):
            (self.records / "registrations" / f"{hashlib.sha256(token.encode()).hexdigest()}.json").write_text(
                json.dumps({"run_id": f"run-{route}", "model": "moonshotai/kimi-k3", "upstream": route,
                            "expires_at": time.time() + 60}),
                encoding="utf-8",
            )
        self.original_connection = relay.http.client.HTTPSConnection
        relay.http.client.HTTPSConnection = FakeUpstreamConnection
        FakeUpstreamConnection.requests = []
        relay.Relay.records = self.records
        relay.Relay.api_keys = {"ai-gateway.vercel.sh": "real-vercel-key", "openrouter.ai": "real-or-key"}
        self.server = ThreadingHTTPServer(("127.0.0.1", 0), relay.Relay)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.http = http.client

    def tearDown(self) -> None:
        self.server.shutdown()
        relay.http.client.HTTPSConnection = self.original_connection
        self.temporary.cleanup()

    def post(self, host: str, token: str, body: dict) -> int:
        connection = self.http.HTTPConnection("127.0.0.1", self.server.server_address[1], timeout=10)
        connection.request("POST", "/v1/chat/completions", body=json.dumps(body),
                           headers={"Host": host, "Authorization": f"Bearer {token}", "Content-Type": "application/json"})
        response = connection.getresponse()
        response.read()
        connection.close()
        return response.status

    def test_vercel_run_is_forwarded_with_real_key_and_charged_cost_recorded(self) -> None:
        FakeUpstreamConnection.body = (
            b'data: {"id":"gen_1","model":"moonshotai/kimi-k3","choices":[{"delta":{"provider_metadata":'
            b'{"gateway":{"cost":"0.03","gatewayCost":"0.0301","generationId":"gen_1"}}}}],'
            b'"usage":{"prompt_tokens":10,"completion_tokens":2,"cost":0.03}}\n\ndata: [DONE]\n\n'
        )
        status = self.post("ai-gateway.vercel.sh", "tok-vercel", {"model": "moonshotai/kimi-k3", "messages": []})
        self.assertEqual(status, 200)
        host, _, path, headers = FakeUpstreamConnection.requests[-1]
        self.assertEqual((host, path, headers["Authorization"]), ("ai-gateway.vercel.sh", "/v1/chat/completions", "Bearer real-vercel-key"))
        usage_path = self.records / "runs" / "run-ai-gateway.vercel.sh" / "usage.jsonl"
        import time
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline and not (usage_path.exists() and usage_path.read_text().strip()):
            time.sleep(0.02)  # accounting is written after the stream closes
        rows = [json.loads(line) for line in usage_path.read_text().splitlines()]
        self.assertEqual(rows[0]["upstream"], "ai-gateway.vercel.sh")
        self.assertEqual(rows[0]["charged_cost_usd"], 0.0301)
        self.assertEqual(rows[0]["generation_ids"], ["gen_1"])

    def test_tokens_are_bound_to_their_gateway_and_model(self) -> None:
        self.assertEqual(self.post("ai-gateway.vercel.sh", "tok-or", {"model": "moonshotai/kimi-k3"}), 401)
        self.assertEqual(self.post("ai-gateway.vercel.sh", "tok-vercel", {"model": "moonshotai/kimi-k2.6"}), 400)
        self.assertEqual(self.post("evil.example", "tok-vercel", {"model": "moonshotai/kimi-k3"}), 421)
        self.assertEqual(FakeUpstreamConnection.requests, [])


class SandboxTests(unittest.TestCase):
    def test_every_harness_gets_same_prompt_native_openrouter_and_no_host_state(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            seen = {}
            for kind in ("corbanu", "hermes", "kilo"):
                argv, env, stdin = sandbox.prepare_agent(kind, "z-ai/glm-5.3", PROMPT, root / kind, "run-token")
                seen[kind] = stdin if stdin is not None else argv[-1]
                self.assertEqual(env["OPENROUTER_API_KEY"], "run-token")
                self.assertEqual(env["SSL_CERT_FILE"], sandbox.CONTAINER_CA)
                self.assertFalse(any(key.startswith(("CORBANU_TASKNODE", "CODEX_THREAD")) for key in env))
                for value in env.values():
                    self.assertFalse(value.startswith(str(Path.home())))
            self.assertEqual(set(seen.values()), {PROMPT})
            vercel_argv, vercel_env, _ = sandbox.prepare_agent(
                "corbanu", "moonshotai/kimi-k3", PROMPT, root / "cv", "t",
                route="vercel", cli_model="vercel/moonshotai/kimi-k3",
            )
            self.assertEqual(vercel_env["AI_GATEWAY_API_KEY"], "t")
            self.assertNotIn("OPENROUTER_API_KEY", vercel_env)
            self.assertIn('model_provider="vercel"', vercel_argv)
            self.assertIn("vercel/moonshotai/kimi-k3", vercel_argv)
            self.assertIn('web_search="disabled"', vercel_argv)
            hermes_argv, _, _ = sandbox.prepare_agent("hermes", "moonshotai/kimi-k3", PROMPT, root / "hv", "t", route="vercel")
            self.assertEqual(hermes_argv[hermes_argv.index("--provider") + 1], "ai-gateway")
            kilo_argv, _, kilo_stdin = sandbox.prepare_agent("kilo", "m", PROMPT, root / "k2", "t")
            self.assertEqual(kilo_stdin, PROMPT)
            self.assertNotIn(PROMPT, kilo_argv)
            self.assertIn('model_provider="openrouter"', " ".join(sandbox.prepare_agent("corbanu", "m", PROMPT, root / "c2", "t")[0]))
            kilo_config = json.loads((root / "kilo" / ".config" / "kilo" / "kilo.json").read_text())
            self.assertEqual(kilo_config["tools"], {"webfetch": False, "websearch": False})
            self.assertIn("web", (root / "hermes" / ".hermes" / "config.yaml").read_text())

    def test_container_mounts_only_workspace_home_and_ca(self) -> None:
        campaign = sandbox.Campaign(sandbox.IsolationSpec(images={"corbanu": "img-c", "kilo": "img-k"}), Path("/runs/x"))
        campaign.relay_ip = "10.0.0.2"
        argv = campaign.container_argv(
            name="n", kind="corbanu", workspace=Path("/runs/x/ws"), home=Path("/runs/x/home"),
            env_file=Path("/runs/x/env"), interactive=True, agent_argv=["corbanu", "exec"],
        )
        mounts = [argv[i + 1] for i, part in enumerate(argv) if part == "--mount"]
        self.assertEqual(
            [mount.split(",")[1] for mount in mounts],
            ["src=/runs/x/ws", "src=/runs/x/home", f"src={campaign.tls / 'ca.pem'}"],
        )
        self.assertIn("openrouter.ai:10.0.0.2", argv)
        self.assertIn("ai-gateway.vercel.sh:10.0.0.2", argv)
        self.assertEqual(argv[argv.index("--network") + 1], campaign.network)
        self.assertNotIn("--privileged", argv)
        self.assertIn("img-c", argv)
        self.assertNotIn("img-k", argv)

    def test_relay_records_summary_prices_and_verifies_route(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            records = Path(temporary)
            rows = [
                {"request_id": "1", "status": 200, "observed_models": ["z-ai/glm-5.3"], "generation_ids": ["gen-1"], "usage": {"cost": 0.25, "prompt_tokens": 10}},
                {"request_id": "2", "status": 200, "observed_models": ["z-ai/glm-5.3-20260816"], "generation_ids": ["gen-2"], "usage": {"cost": 0.5, "prompt_tokens": 20}},
            ]
            (records / "usage.jsonl").write_text("\n".join(json.dumps(r) for r in rows), encoding="utf-8")
            (records / "events.jsonl").write_text(json.dumps({"decision": "rejected", "reason": "model mismatch"}), encoding="utf-8")
            summary = sandbox.summarize_run_records(records, "z-ai/glm-5.3")
            self.assertTrue(summary["route_verified"])
            self.assertEqual(summary["relay_cost_usd"], 0.75)
            self.assertEqual(summary["rejected_requests"], 1)
            self.assertEqual(summary["usage_totals"]["prompt_tokens"], 30)
            other = sandbox.summarize_run_records(records, "moonshotai/kimi-k3")
            self.assertFalse(other["route_verified"])
        self.assertTrue(sandbox._same_model("global.moonshotai.kimi-k3", "moonshotai/kimi-k3"))
        self.assertTrue(sandbox._same_model("moonshotai/Kimi-K3", "moonshotai/kimi-k3"))
        self.assertFalse(sandbox._same_model("moonshotai/kimi-k2.6", "moonshotai/kimi-k3"))


class RunnerIsolationTests(unittest.TestCase):
    def test_isolated_run_root_must_be_outside_git_checkouts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "repo" / ".git").mkdir(parents=True)
            with self.assertRaises(RuntimeError):
                runner.ensure_outside_repository(root / "repo" / "runs" / "x")
            runner.ensure_outside_repository(root / "elsewhere" / "runs")

    def test_run_ids_are_container_safe(self) -> None:
        task = runner.TaskSpec("rate gate", Path("/b"), Path("/p"), Path("/v"), 10, ("python",))
        agent = runner.AgentSpec("kilo/opus", "kilo", "kilo", "openrouter", "m", "lane", ())
        self.assertEqual(runner.isolated_run_id(task, agent, 3), "rate_gate--kilo_opus--w003")

    def test_campaign_configs_are_isolated_and_prompt_only(self) -> None:
        for name in ("openrouter-three-harness.json", "openrouter-smoke.json"):
            config = json.loads((BENCH_ROOT / "configs" / name).read_text())
            self.assertEqual(config["isolation"]["mode"], "docker")
            images = config["isolation"]["images"]
            self.assertEqual(sorted(images), ["corbanu", "hermes", "kilo"])
            self.assertEqual(len(set(images.values())), 3)
            self.assertNotIn("toothpaste-site", [task["name"] for task in config["tasks"]])
            for agent in config["agents"]:
                self.assertNotIn("command", agent)
                self.assertEqual(agent["provider"], "openrouter")


if __name__ == "__main__":
    unittest.main()

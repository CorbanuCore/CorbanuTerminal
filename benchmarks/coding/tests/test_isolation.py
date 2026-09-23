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

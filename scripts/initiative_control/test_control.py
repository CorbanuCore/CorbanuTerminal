import copy
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import urllib.error
import urllib.request

import control
import delivery
import tasknode
import activate
import tick


def run():
    return dict(run_id="run-1", sprint_id="PF-80-S01", machine="synthetic-machine",
                role="tester", agent="synthetic", session_id="native-1", status="working",
                summary="Recovery test passed; human acceptance still pending.", updated_at=control.now(),
                commit="a" * 40, branch="test/fixture", worktree="/synthetic/worktree")


class ControlTests(unittest.TestCase):
    def test_run_retry_is_idempotent_and_private(self):
        with tempfile.TemporaryDirectory() as tmp:
            state, value = Path(tmp), run()
            self.assertEqual(control.report(state, value), control.report(state, value))
            self.assertEqual(len(list((state / "events").glob("*.json"))), 1)
            self.assertEqual(next((state / "events").iterdir()).stat().st_mode & 0o077, 0)

    def test_extra_fields_never_become_raw_log_uploads(self):
        with self.assertRaises(ValueError):
            control.checked_run({**run(), "raw_transcript": "unwanted"})

    def test_sensitive_summaries_are_rejected(self):
        for text in ("password=synthetic-canary", "Bearer synthetic-canary", "api_key: synthetic", "\x1b[31m"):
            with self.subTest(text=text), self.assertRaises(ValueError):
                control.checked_run({**run(), "summary": text})

    def test_future_report_and_bad_identity_rejected(self):
        for fields in ({"updated_at": "2099-01-01T00:00:00Z"}, {"run_id": "../escape"}, {"commit": "HEAD"}, {"status": "accepted"}, {"updated_at": "2026-09-10T10:00:00"}):
            with self.subTest(fields=fields), self.assertRaises(ValueError):
                control.checked_run({**run(), **fields})

    def test_unsafe_markdown_does_not_execute_or_load_images(self):
        result = control.safe_markdown('<script>alert(1)</script>\n\n![x](https://example.com/track)\n\n[x](javascript:alert(1))\n\n[private](file:///etc/passwd)\n\n[secret](https://example.com/?key=123)', "docs/plan.md", {})
        self.assertNotIn("<script>", result)
        self.assertNotIn("<img", result)
        self.assertNotIn('href="javascript:', result)
        self.assertNotIn('href="file:', result)
        self.assertNotIn("?key=123", result)

    def test_markdown_tables_and_relative_document_links(self):
        result = control.safe_markdown("| A | B |\n|---|---|\n| one | two |\n\n[Next](../sprint.md)", "docs/plans/a.md", {"docs/sprint.md": ""})
        self.assertIn("<table>", result)
        self.assertIn(control.route("docs/sprint.md"), result)

    def test_no_path_escape_or_symlink_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            inside = root / "source"
            inside.mkdir()
            (root / "secret").write_text("synthetic")
            (inside / "link").symlink_to(root / "secret")
            for path in (inside / "../secret", inside / "link"):
                with self.assertRaises(ValueError):
                    control.read_file(path, inside)

    def test_oversized_source_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "large"
            path.write_text("x" * (control.MAX_FILE + 1))
            with self.assertRaises(ValueError):
                control.read_file(path, Path(tmp))

    def test_facilities_page_indexes_all_machine_interfaces(self):
        with patch.object(socket, "create_connection", side_effect=AssertionError("static index must not probe services")):
            body = control.facilities()
        for name, endpoint in (("ComfyUI", "http://100.99.88.49:8188/"),
                               ("YuE2 (YuE)", "http://100.99.88.49:7861/"),
                               ("ACE-Step", "http://100.99.88.49:7862/"),
                               ("MiniMax Music 3", "http://100.99.88.49:7863/"),
                               ("RVC", "http://100.81.145.102:7865/"),
                               ("ACE-Step fallback", "http://100.81.145.102:7866/")):
            self.assertIn(name, body)
            self.assertEqual(body.count(f'href="{endpoint}"'), 3)  # title, action, table
        for repository in ("comfyanonymous/ComfyUI", "multimodal-art-projection/YuE",
                           "ace-step/ACE-Step-1.5", "RVC-Project/Retrieval-based-Voice-Conversion-WebUI"):
            self.assertIn(f'href="https://github.com/{repository}"', body)
        self.assertIn('href="https://huggingface.co/MiniMaxAI/MiniMax-Music3"', body)
        self.assertIn("6 registered interfaces", body)
        self.assertEqual(body.count('<article class="test facility-card"'), 6)
        self.assertEqual(body.count('data-facility-action="start"'), 6)
        self.assertEqual(body.count('data-facility-action="stop"'), 6)
        self.assertIn('data-control-endpoint="http://127.0.0.1:8770"', body)
        self.assertIn("Live service status", body)
        self.assertIn("At a glance", body)
        self.assertIn("An unavailable machine remains unavailable", body)

    def test_top_navigation_links_facilities(self):
        for title in ("Initiative map", "Facilities", "Sprint document"):
            self.assertIn('href="facilities.html">Facilities</a>', control.page(title, "", control.now()))

    def test_facilities_published_atomically_and_failure_keeps_previous_page(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            data = dict(documents={}, source={"collected_at": control.now()}, runs=[], problems=[])
            with patch.object(control, "collect", return_value=data), patch.object(control, "overview", return_value="map"):
                control.publish(root, root, root)
                current = (root / "current").resolve()
                body = (root / "current/facilities.html").read_text()
                self.assertIn("Facilities", body)
                for asset in ("style.css", "facilities.css", "status.js", "facilities.js"):
                    self.assertTrue((root / "current" / asset).is_file())
                self.assertIn(f'data-generation="{current.name}"', body)
                with patch.object(control, "facilities", side_effect=ValueError("fixture failure")):
                    with self.assertRaises(ValueError):
                        control.publish(root, root, root)
                self.assertEqual((root / "current").resolve(), current)
                self.assertEqual((root / "current/facilities.html").read_text(), body)

    def test_failed_publication_retains_old_generation(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            old = root / "old"
            old.mkdir()
            (old / "index.html").write_text("last-good")
            (root / "current").symlink_to(old)
            with patch.object(control, "collect", side_effect=ValueError("bad source")):
                with self.assertRaises(ValueError):
                    control.publish(root, root, root)
            self.assertEqual((root / "current/index.html").read_text(), "last-good")
            self.assertFalse(json.loads((root / "health.json").read_text())["ok"])

    def test_status_js_checks_source_age_not_just_publish_age(self):
        text = (control.HERE / "status.js").read_text()
        self.assertIn("document.body.dataset.collected", text)
        self.assertIn("publishAge > 45", text)
        self.assertIn("document.body.dataset.generation", text)

    def test_partial_render_is_removed_without_touching_last_good(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            old = root / "old"
            old.mkdir()
            (root / "current").symlink_to(old)
            data = {"documents": {"test.md": "# Fixture"}, "source": {"collected_at": control.now()}}
            with patch.object(control, "collect", return_value=data), patch.object(control, "overview", side_effect=ValueError("bad metadata")):
                with self.assertRaises(ValueError):
                    control.publish(root, root, root)
            self.assertEqual(list((root / "releases").iterdir()), [])
            self.assertEqual((root / "current").resolve(), old.resolve())

    def test_writeback_panel_reports_real_state_and_missing_state(self):
        data = dict(plans={"plans": [], "active_limit": 3}, sprints={"sprints": []}, config={"tasknode": {"enabled": False, "status": "Misleading manager label"}},
                    source={"label": "fixture", "branch": "fixture", "commit": "a" * 40}, problems=[], runs=[], events=[])
        body = control.overview(data)
        self.assertIn("Live delivery disabled", body)
        self.assertIn("Delivery state unknown", body)
        self.assertNotIn("Misleading manager label", body)
        data["writeback"] = {"checked_at": control.now(), "outbox": {"blocked": 2}, "error": "Operator recovery required"}
        body = control.overview(data)
        self.assertIn("blocked 2", body)
        self.assertIn("Operator recovery required", body)

    def test_clock_rollback_cannot_prune_current_and_abandoned_builds_expire(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            releases = root / "releases"
            for i in range(4):
                path = releases / f"build-{i:08d}"
                path.mkdir(parents=True)
                if i < 3:
                    control.atomic_json(path / "manifest.json", {})
                os.utime(path, (4102444800 if i < 3 else 1000,) * 2)
            data = {"documents": {}, "source": {"collected_at": control.now()}, "runs": [], "problems": []}
            with patch.object(control, "collect", return_value=data), patch.object(control, "overview", return_value="fixture"):
                control.publish(root, root, root)
            self.assertTrue((root / "current/index.html").is_file())
            self.assertEqual(len(list(releases.iterdir())), 3)
            self.assertFalse((releases / "build-00000003").exists())


class OutboxTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.state = Path(self.tmp.name)
        self.config = {"tasknode": {"enabled": True, "workspace_id": "workspace-1", "task_mappings": {"PF-80-S01": ["owned-task-1"]}}}
        control.atomic_json(self.state / "control.json", self.config)
        control.atomic_json(self.state / "enrollment.json", {"verified": True, "workspace_id": "workspace-1", "verified_at": control.now()})

    def record(self, event_id):
        return control.read_json(self.state / "outbox" / (event_id + ".json"), self.state)

    def ok(self, _path, body, _auth):
        return 200, {"ok": True, "id": body["event"]["id"], "summaryState": "not_applicable"}

    def test_exact_payload_survives_timeout_and_retry(self):
        value = run()
        event_id = tasknode.enqueue(self.state, value)
        self.assertEqual(tasknode.enqueue(self.state, value), event_id)
        before = copy.deepcopy(self.record(event_id)["event"])
        self.assertEqual(tasknode.flush(self.state, {}, lambda *_: (0, {})), 0)
        record = self.record(event_id)
        record["next_attempt_at"] = "2020-01-01T00:00:00Z"
        control.atomic_json(self.state / "outbox" / (event_id + ".json"), record)
        self.assertEqual(tasknode.flush(self.state, {}, self.ok), 1)
        self.assertEqual(self.record(event_id)["event"], before)
        self.assertEqual(tasknode.flush(self.state, {}, self.ok), 0)

    def test_only_goal_metadata_not_paid_output_or_completion(self):
        value = tasknode.event_for(run(), "workspace", ["task"], 0)
        self.assertEqual(value["kind"], "goal")
        self.assertEqual(value["coverage"], "manager_observed_worker_report_not_independent_acceptance")
        self.assertNotIn("worktree", json.dumps(value))
        self.assertTrue(value["occurredAt"].endswith("Z"))

    def test_live_delivery_requires_explicit_switch(self):
        self.config["tasknode"]["enabled"] = False
        control.atomic_json(self.state / "control.json", self.config)
        with self.assertRaises(ValueError):
            tasknode.flush(self.state, {}, self.ok)

    def test_unmapped_report_cannot_be_enqueued(self):
        self.config["tasknode"]["task_mappings"] = {}
        control.atomic_json(self.state / "control.json", self.config)
        with self.assertRaises(ValueError):
            tasknode.enqueue(self.state, run())

    def test_unauthorized_conflict_and_wrong_response_id_block(self):
        for code in (401, 403, 409, 400, 200):
            value = {**run(), "run_id": f"run-{code}"}
            event_id = tasknode.enqueue(self.state, value)
            tasknode.flush(self.state, {}, lambda *_: (code, {"ok": True, "id": "wrong"}))
            self.assertEqual(self.record(event_id)["status"], "blocked")
            old = self.record(event_id)["event"]
            tasknode.retry(self.state, event_id)
            self.assertEqual(self.record(event_id)["event"], old)
            self.assertEqual(self.record(event_id)["status"], "pending")

    def test_changed_mapping_cannot_retarget_queued_event(self):
        event_id = tasknode.enqueue(self.state, run())
        self.config["tasknode"]["task_mappings"]["PF-80-S01"] = ["other-task"]
        control.atomic_json(self.state / "control.json", self.config)
        with patch.object(tasknode, "post", side_effect=AssertionError("network must not run")):
            tasknode.flush(self.state, {})
        self.assertEqual(self.record(event_id)["status"], "blocked")

    def test_delivered_records_do_not_starve_later_pending_work(self):
        for i in range(25):
            tasknode.enqueue(self.state, {**run(), "run_id": f"run-{i}"})
        self.assertEqual(tasknode.flush(self.state, {}, self.ok), 20)
        self.assertEqual(tasknode.flush(self.state, {}, self.ok), 5)

    def test_redirect_is_not_followed(self):
        self.assertIsNone(tasknode.NoRedirect().redirect_request(None, None, 302, "", {}, "https://evil.invalid"))

    def test_multibyte_limit_is_enforced(self):
        with self.assertRaises(ValueError):
            tasknode.event_for({**run(), "summary": "猫" * 800}, "workspace", ["task"], 0)

    def test_enrollment_is_local_and_workspace_bound(self):
        tasknode.enroll(self.state, {}, lambda *_: (200, {"ok": True}))
        self.assertNotIn("enrollment_verified", control.read_json(self.state / "control.json", self.state)["tasknode"])
        self.config["tasknode"]["workspace_id"] = "different-workspace"
        self.config["tasknode"]["enrollment_verified"] = True
        control.atomic_json(self.state / "control.json", self.config)
        with self.assertRaises(ValueError):
            tasknode.flush(self.state, {}, self.ok)

    def test_non_object_http_response_is_a_retryable_failure(self):
        with patch.object(tasknode.urllib.request, "build_opener") as build:
            build.return_value.open.return_value.__enter__.return_value.read.return_value = b"[]"
            self.assertEqual(tasknode.post("/events", {"event": {}}, {"terminal_session": "synthetic", "api_key": "synthetic"}), (0, {}))


class RefreshTests(unittest.TestCase):
    def test_unexpected_tick_failure_marks_health_failed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            control.atomic_json(root / "state/control.json", {})
            control.atomic_json(root / "site/health.json", {"ok": True})
            with patch.object(tick, "collect", return_value={"runs": []}):
                with self.assertRaises(KeyError):
                    tick.tick(root)
            self.assertFalse(control.read_json(root / "site/health.json", root)["ok"])

    def test_rejected_report_does_not_starve_next_run_and_status_is_published(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = root / "state"
            control.atomic_json(state / "control.json", {"tasknode": {"enabled": False, "workspace_id": "workspace", "task_mappings": {"PF-80-S01": ["task"]}}})
            data = {"runs": [{**run(), "summary": "猫" * 800}, {**run(), "run_id": "next-run"}]}
            def published(*_):
                status = control.read_json(state / "writeback-status.json", state)
                self.assertEqual(status["rejected_runs"], 1)
                self.assertEqual(status["outbox"], {"pending": 1})
                self.assertIn("rejected", status["error"])
            with patch.object(tick, "collect", return_value=data), patch.object(tick, "publish", side_effect=published) as publisher, patch.object(tick, "flush", side_effect=AssertionError("disabled")):
                tick.tick(root)
            publisher.assert_called_once()

    def test_sync_preserves_enrollment_and_prunes_old_partial_uploads(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "corbanu-control"
            root.mkdir()
            receipt = {"verified": True, "workspace_id": "workspace", "verified_at": control.now()}
            control.atomic_json(root / "state/enrollment.json", receipt)
            uploads = []
            for i in range(5):
                incoming = root / "incoming" / ("upload-" + f"{i:032x}")
                (incoming / "source").mkdir(parents=True)
                os.utime(incoming, (1000 + i, 1000 + i))
                uploads.append(incoming)
            final = uploads[-1]
            control.atomic_json(final / "state/source.json", {"files": {}})
            control.atomic_json(final / "state/control.json", {"tasknode": {"enabled": False}})
            with patch.object(activate.subprocess, "run"):
                activate.activate(root, final, root / "units")
            self.assertEqual(control.read_json(root / "state/enrollment.json", root), receipt)
            self.assertEqual(len(list((root / "incoming").iterdir())), 3)
            self.assertEqual((root / "source").resolve(), (final / "source").resolve())
            self.assertIn(sys.executable, (root / "units/corbanu-control-web.service").read_text())

    def test_invalid_import_cannot_replace_source_manifest_or_config(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve() / "corbanu-control"
            original = root / "original"
            original.mkdir(parents=True)
            (root / "source").symlink_to(original)
            control.atomic_json(root / "state/source.json", {"old": True})
            control.atomic_json(root / "state/control.json", {"old": True})
            incoming = root / "incoming" / ("upload-" + "0" * 32)
            control.atomic_json(incoming / "state/source.json", {"files": {}})
            control.atomic_json(incoming / "state/control.json", {"new": True})
            control.atomic_json(incoming / "state/events/bad.json", {**run(), "updated_at": "2099-01-01T00:00:00Z"})
            with self.assertRaises(ValueError), patch.object(activate.subprocess, "run", side_effect=AssertionError("no services before validation")):
                activate.activate(root, incoming, root / "units")
            self.assertEqual(control.read_json(root / "state/source.json", root), {"old": True})
            self.assertEqual(control.read_json(root / "state/control.json", root), {"old": True})
            self.assertEqual((root / "source").resolve(), original)

    def test_real_http_server_is_read_only_and_rejects_untrusted_hosts_and_paths(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "current").mkdir()
            (root / "current/index.html").write_text("<h1>Synthetic fixture</h1>")
            facility_body = control.page("Facilities", control.facilities(), control.now())
            (root / "current/facilities.html").write_text(facility_body)
            control.atomic_json(root / "health.json", {"ok": True})
            with socket.socket() as sock:
                sock.bind(("127.0.0.1", 0))
                port = sock.getsockname()[1]
            process = subprocess.Popen([sys.executable, str(control.HERE / "control.py"), "serve", "--output", str(root), "--port", str(port)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                base = f"http://127.0.0.1:{port}"
                for _ in range(100):
                    try:
                        with urllib.request.urlopen(base, timeout=1) as response:
                            self.assertEqual(response.headers["X-Corbanu-Control"], "1")
                            self.assertIn("connect-src 'self' http://127.0.0.1:8770", response.headers["Content-Security-Policy"])
                            self.assertIn("frame-ancestors 'none'", response.headers["Content-Security-Policy"])
                            self.assertIn("Synthetic fixture", response.read().decode())
                            break
                    except urllib.error.URLError:
                        time.sleep(0.02)
                else:
                    self.fail("fixture server did not start")
                for method in ("GET", "HEAD"):
                    request = urllib.request.Request(base + "/facilities.html", method=method)
                    with urllib.request.urlopen(request, timeout=1) as response:
                        self.assertEqual(response.status, 200)
                        self.assertEqual(response.headers["X-Corbanu-Control"], "1")
                        self.assertEqual(int(response.headers["Content-Length"]), len(facility_body.encode()))
                        self.assertEqual(response.read(), b"" if method == "HEAD" else facility_body.encode())
                    for path, host, code in (("/facilities.html", "evil.invalid", 403),
                                             ("/%2e%2e/control.json", "127.0.0.1", 404),
                                             ("/state/control.json", "127.0.0.1", 404),
                                             ("/facilitiesXhtml", "127.0.0.1", 404)):
                        request = urllib.request.Request(base + path, method=method, headers={"Host": host})
                        with self.assertRaises(urllib.error.HTTPError) as caught:
                            urllib.request.urlopen(request, timeout=1)
                        self.assertEqual(caught.exception.code, code)
                        caught.exception.close()
                cases = [urllib.request.Request(base, headers={"Host": "evil.invalid"}),
                         urllib.request.Request(base + "/%2e%2e/control.json"),
                         urllib.request.Request(base + "/state/control.json"),
                         urllib.request.Request(base, data=b"no writes", method="POST")]
                for request, code in zip(cases, (403, 404, 404, 501)):
                    with self.subTest(code=code), self.assertRaises(urllib.error.HTTPError) as caught:
                        urllib.request.urlopen(request, timeout=1)
                    self.assertEqual(caught.exception.code, code)
                    caught.exception.close()
            finally:
                process.terminate()
                process.wait(timeout=5)


class DeliveryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        registry = self.repo / "codex-rs/features/src/lib.rs"
        registry.parent.mkdir(parents=True)
        registry.write_text('FeatureSpec {\n        key: "test_flag",\n        stage: Stage::UnderDevelopment,\n        default_enabled: false,\n    },')
        (self.repo / "evidence.txt").write_text("Synthetic test evidence only")
        candidate = delivery.digest(self.repo, ["codex-rs/features/src/lib.rs"])
        evidence = hashlib.sha256((self.repo / "evidence.txt").read_bytes()).hexdigest()
        self.contract = dict(mode="feature-flagged", flag="test_flag", candidate_files=["codex-rs/features/src/lib.rs"], candidate_tree_sha256=candidate,
                             boundaries={k: "A specific synthetic gate" for k in delivery.BOUNDARIES}, tests={k: dict(result="pass", candidate_tree_sha256=candidate, artifact="evidence.txt", artifact_sha256=evidence) for k in delivery.ENABLE_CASES})

    def test_merge_pass_does_not_imply_enablement(self):
        self.assertEqual(delivery.check(self.repo, self.contract, "merge"), [])
        self.assertTrue(delivery.check(self.repo, self.contract, "enable"))

    def test_default_on_or_public_stage_cannot_merge(self):
        self.contract["flag"] = "missing"
        self.assertTrue(delivery.check(self.repo, self.contract, "merge"))
        self.contract["flag"] = "test_flag"
        path = self.repo / "codex-rs/features/src/lib.rs"
        path.write_text(path.read_text().replace("false", "true"))
        self.assertTrue(delivery.check(self.repo, self.contract, "merge"))

    def test_stale_candidate_and_artifact_rejected(self):
        (self.repo / "evidence.txt").write_text("Changed")
        self.assertTrue(any("evidence artifact changed" in x for x in delivery.check(self.repo, self.contract, "merge")))

    def test_all_recovery_cases_required_for_enable(self):
        self.contract["human_acceptance"] = dict(tester="Synthetic tester", result="pass", candidate_tree_sha256=self.contract["candidate_tree_sha256"])
        self.contract["enablement_decision"] = "Synthetic fixture, not a product approval"
        self.assertEqual(delivery.check(self.repo, self.contract, "enable"), [])
        for case in delivery.ENABLE_CASES:
            contract = copy.deepcopy(self.contract)
            del contract["tests"][case]
            self.assertTrue(delivery.check(self.repo, contract, "enable"), case)


if __name__ == "__main__":
    unittest.main()

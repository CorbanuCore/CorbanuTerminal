"""Synthetic preparation/migration tests. Never load operator state or credentials."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import control
import export
import tasknode
import tick
from test_control import run


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.state = Path(self.tmp.name)
        self.config = {"tasknode": {"workspace_id": "synthetic-workspace",
                       "task_mappings": {"PF-80-S01": ["synthetic-owned-task"]}}}
        control.atomic_json(self.state / "control.json", self.config)
        self.event_id = tasknode.enqueue(self.state, run())

    def snapshot(self):
        return {p.relative_to(self.state).as_posix(): p.read_bytes()
                for p in self.state.rglob("*") if p.is_file()}

    def record(self):
        return control.read_json(self.state / "outbox" / (self.event_id + ".json"), self.state)

    def save(self, record):
        control.atomic_json(self.state / "outbox" / (self.event_id + ".json"), record)

    def enable_fixture(self):
        self.config["tasknode"]["enabled"] = True
        control.atomic_json(self.state / "control.json", self.config)
        control.atomic_json(self.state / "enrollment.json", {"verified": True,
                            "workspace_id": "synthetic-workspace"})

    def test_preparation_is_read_only_and_default_off_even_with_batch(self):
        for i in range(24):
            tasknode.enqueue(self.state, {**run(), "run_id": f"synthetic-{i}"})
        before = self.snapshot()
        with patch.object(tasknode, "post", side_effect=AssertionError("no network")), \
             patch.object(tasknode, "credentials", side_effect=AssertionError("no auth")), \
             patch.object(tasknode, "flush", side_effect=AssertionError("no batch")), \
             patch.object(tasknode.urllib.request, "build_opener", side_effect=AssertionError("no HTTP")):
            result = tasknode.prepare(self.state, self.event_id)
        self.assertEqual(result["payload"], {"event": self.record()["event"]})
        self.assertEqual(result["selected_count"], 1)
        self.assertFalse(result["network_writes"])
        self.assertFalse(result["send_authorized"])
        self.assertIn("posting_disabled", result["blockers"])
        self.assertIn("local_workspace_enrollment_unverified", result["blockers"])
        self.assertEqual(self.snapshot(), before)

    def test_even_locally_enabled_enrolled_preparation_has_no_send_authority(self):
        self.enable_fixture()
        before = self.snapshot()
        with patch.object(tasknode, "post", side_effect=AssertionError("no network")):
            result = tasknode.prepare(self.state, self.event_id)
        self.assertTrue(result["local_workspace_enrollment"])
        self.assertFalse(result["send_authorized"])
        self.assertEqual(result["blockers"], ["live_authority_entitlement_owner_and_target_lifecycle_unverified"])
        self.assertEqual(self.snapshot(), before)

    def test_single_selection_does_not_read_unselected_malformed_record(self):
        (self.state / "outbox" / ("cc-" + "0" * 64 + ".json")).write_text("malformed")
        before = self.snapshot()
        self.assertEqual(tasknode.prepare(self.state, self.event_id)["event_id"], self.event_id)
        self.assertEqual(self.snapshot(), before)

    def test_no_selection_fallback_for_missing_invalid_or_multiple_ids(self):
        before = self.snapshot()
        for value in (None, "", "../control", "cc-" + "0" * 64, self.event_id + "," + self.event_id):
            with self.subTest(value=value), self.assertRaises((ValueError, FileNotFoundError)):
                tasknode.prepare(self.state, value)
        self.assertEqual(self.snapshot(), before)

    def test_symlink_selection_cannot_escape_state(self):
        path = self.state / "outbox" / (self.event_id + ".json")
        path.unlink()
        path.symlink_to(self.state / "control.json")
        with self.assertRaises(ValueError):
            tasknode.prepare(self.state, self.event_id)

    def test_payload_change_is_rejected_before_preview_or_transport(self):
        self.enable_fixture()
        record = self.record()
        record["event"]["content"] = "Changed after enqueue"
        self.save(record)
        with self.assertRaisesRegex(ValueError, "payload changed"):
            tasknode.prepare(self.state, self.event_id)
        with patch.object(tasknode, "post", side_effect=AssertionError("no network")):
            self.assertEqual(tasknode.flush(self.state, {}), 0)
        self.assertEqual(self.record()["error"], "immutable_event_invalid")
        with self.assertRaises(ValueError):
            tasknode.retry(self.state, self.event_id)

    def test_extra_raw_fields_are_rejected_before_hash_validation(self):
        event = self.record()["event"]
        event["raw_transcript"] = "synthetic private text"
        with self.assertRaisesRegex(ValueError, "fields"):
            tasknode.checked_event(event, self.event_id)

    def test_mapping_and_workspace_changes_stay_visible_without_retargeting(self):
        self.config["tasknode"].update(workspace_id="different", task_mappings={"PF-80-S01": ["another"]})
        control.atomic_json(self.state / "control.json", self.config)
        before = self.snapshot()
        result = tasknode.prepare(self.state, self.event_id)
        self.assertIn("mapping_or_workspace_changed", result["blockers"])
        self.assertEqual(result["payload"]["event"]["taskIds"], ["synthetic-owned-task"])
        self.assertEqual(self.snapshot(), before)

    def test_wrong_workspace_or_forged_config_enrollment_is_not_accepted(self):
        self.config["tasknode"]["enrollment_verified"] = True
        control.atomic_json(self.state / "control.json", self.config)
        control.atomic_json(self.state / "enrollment.json", {"verified": True, "workspace_id": "other"})
        self.assertFalse(tasknode.prepare(self.state, self.event_id)["local_workspace_enrollment"])

    def test_disabled_restart_does_not_read_credentials_or_send(self):
        self.enable_fixture()
        self.config["tasknode"]["enabled"] = False
        control.atomic_json(self.state / "control.json", self.config)
        before = self.snapshot()
        with patch.object(tasknode, "post", side_effect=AssertionError("no send")):
            with self.assertRaises(ValueError):
                tasknode.flush(self.state, {})
        with patch.object(tick, "collect", return_value={"runs": []}), \
             patch.object(tick, "publish"), \
             patch.object(tick, "credentials", side_effect=AssertionError("no credentials")), \
             patch.object(tick, "flush", side_effect=AssertionError("no batch")):
            root = self.state / "service"
            control.atomic_json(root / "state/control.json", self.config)
            control.atomic_json(root / "state/outbox" / (self.event_id + ".json"), self.record())
            tick.refresh(root)
            self.assertEqual(control.read_json(root / "state/outbox" / (self.event_id + ".json"), root), self.record())
        for name, value in before.items():
            self.assertEqual((self.state / name).read_bytes(), value)

    def test_retry_and_delivered_status_are_never_reset_by_preparation(self):
        for status in ("pending", "blocked", "delivered"):
            record = self.record()
            record.update(status=status, attempts=3, next_attempt_at="2099-01-01T00:00:00Z")
            self.save(record)
            before = self.snapshot()
            result = tasknode.prepare(self.state, self.event_id)
            self.assertIn("retry_backoff_active" if status == "pending" else "record_not_pending", result["blockers"])
            self.assertEqual(self.snapshot(), before)

    def test_cli_preview_and_prepare_reject_all_live_arguments(self):
        before = self.snapshot()
        for command in ("prepare", "preview"):
            args = [sys.executable, str(control.HERE / "tasknode.py"), command,
                    "--state", str(self.state), "--event-id", self.event_id]
            result = subprocess.run(args, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(json.loads(result.stdout)["send_authorized"])
            for extra in (["--confirm-live"], ["--credentials-file", "/synthetic/must-not-read"], ["--report", "/synthetic/report"]):
                rejected = subprocess.run(args + extra, capture_output=True, text=True)
                self.assertEqual(rejected.returncode, 2)
                self.assertIn("live/report arguments forbidden", rejected.stderr)
        self.assertEqual(self.snapshot(), before)

    def test_cli_flush_cannot_silently_ignore_single_event_selection(self):
        result = subprocess.run([sys.executable, str(control.HERE / "tasknode.py"), "flush",
                                "--state", str(self.state), "--event-id", self.event_id,
                                "--confirm-live", "--credentials-file", "/synthetic/must-not-read"],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertIn("flush is a batch operation", result.stderr)


class MigrationTests(unittest.TestCase):
    def test_historical_batch_has_no_remap_requeue_or_delivery(self):
        fixture = json.loads((control.HERE / "fixtures/recovery-old-state.json").read_text())
        with tempfile.TemporaryDirectory() as tmp:
            state = Path(tmp)
            control.atomic_json(state / "control.json", {"tasknode": {"enabled": True,
                "workspace_id": "synthetic-workspace", "task_mappings": {
                    "PF-80-S01": ["synthetic-delivery-task"],
                    "PF-76-S01": ["synthetic-delivery-task"]}}})
            control.atomic_json(state / "enrollment.json", {"verified": True, "workspace_id": "synthetic-workspace"})
            for sequence in range(25):
                event = tasknode.event_for(fixture["report"], "synthetic-workspace", ["synthetic-delivery-task"], sequence)
                control.atomic_json(state / "outbox" / (event["id"] + ".json"),
                                    {"event": event, "status": "pending", "attempts": 0, "next_attempt_at": control.now()})
            paths = list((state / "outbox").glob("*.json"))
            before = {p.name: p.read_bytes() for p in paths}
            selected = paths[0].stem
            result = tasknode.prepare(state, selected)
            self.assertIn("historical_source_reconciliation_required", result["blockers"])
            self.assertIn("stale_observation_requires_review", result["blockers"])
            self.assertEqual(result["payload"]["event"]["turnId"], "PF-76-S01")
            with patch.object(tasknode, "post", side_effect=AssertionError("no history delivery")):
                self.assertEqual(tasknode.flush(state, {}), 0)
            with self.assertRaises(ValueError):
                tasknode.retry(state, selected)
            with self.assertRaises(ValueError):
                tasknode.enqueue(state, fixture["report"])
            self.assertEqual({p.name: p.read_bytes() for p in paths}, before)
            self.assertFalse((state / "outbox-index.json").exists())

    def test_receiving_tree_publication_holds_ambiguous_history_and_detects_source_drift(self):
        repo = control.HERE.parents[1]
        fixture = json.loads((control.HERE / "fixtures/recovery-old-state.json").read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state, output = root / "state", root / "site"
            control.atomic_json(state / "control.json", {"human_tests": [], "tasknode": {"enabled": False}})
            control.report(state, fixture["report"])
            control.report(state, run())
            exported = root / "export"
            manifest = export.export(repo, state, exported)
            data = control.publish(exported / "source", state, output)
            self.assertEqual([r["sprint_id"] for r in data["runs"]], ["PF-80-S01"])
            self.assertTrue(any("PF-76-S01 history held" in p for p in data["problems"]))
            self.assertEqual(data["plans"]["errors"], [])
            self.assertEqual(data["sprints"]["errors"], [])
            self.assertGreater(len(manifest["files"]), 200)
            self.assertEqual(len(list((state / "events").glob("*.json"))), 2)
            self.assertNotIn("recovery-synthetic-1", (output / "current/index.html").read_text())
            old = (output / "current").resolve()
            (exported / "source/docs/plans/index.md").write_text("malformed replacement")
            with self.assertRaises(ValueError):
                control.publish(exported / "source", state, output)
            self.assertEqual((output / "current").resolve(), old)
            self.assertFalse(control.read_json(output / "health.json", output)["ok"])


if __name__ == "__main__":
    unittest.main()

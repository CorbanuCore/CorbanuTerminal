"""Disposable evidence-gate fixtures; none of these records grants live authority."""
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
from unittest.mock import patch

import activate
import fable_launcher as f
import owner_daemon as owner
from coordinator import digest
from owner_tmux import freeze_worker_inputs
from test_coordinator import seed
import test_owner_daemon as owner_tests
import owner_promotion_94_preflight as gate


def fixture_manifest(evidence, state, old, new, transport):
    """Synthetic declarations ONLY, for testing validator mechanics."""
    ref = dict(path=str(evidence), sha256=f.file_digest(evidence))
    return dict(
        preparation=dict(bridge="owner_tmux.freeze_worker_inputs", provider="fixture",
                         policy="--yolo", authority=ref, adoption=ref),
        replacements={new: dict(old_action=old, evidence=ref)},
        qualification=dict(mode="qualified", evidence=ref, review=ref,
                           binary_sha256=transport["binary_sha256"],
                           package_digest=owner.package_digest(),
                           isolated_transport=True, isolated_profile=True,
                           negative_access_probes=True, mediated_inference=True,
                           real_ack=1, real_start=1, real_return=1,
                           independent_reviewer="SYNTHETIC fixture; no real qualification"),
        audit=dict(dispatchers_quiescent=True, raw_key_delivery_stopped=True,
                   publishers_quiescent=True, observed_at=time.time(),
                   coordinator_revision=state["revision"], evidence=ref),
        activation=dict(decision_id="synthetic-promotion", revision=1,
                        authority="disposable rehearsal only"),
        activation_authority=ref)


class PromotionTests(unittest.TestCase):
    setUp = owner_tests.RecurrenceTests.setUp
    tearDown = owner_tests.RecurrenceTests.tearDown
    installation = owner_tests.RecurrenceTests.installation
    install = owner_tests.RecurrenceTests.install
    arm = owner_tests.RecurrenceTests.arm
    sql = owner_tests.RecurrenceTests.sql

    def prepare(self):
        self.args = self.installation()
        service, command, self.calls = self.install(self.args)
        self.enterContext(service)
        self.enterContext(command)
        activate.owner_activation(self.args)
        self.arm()
        base = Path(self.tmp.name)
        brief = base / "brief.json"
        f.write_json(brief, {"task": "fixture only"})
        inputs = dict(allocation="promotion", base_commit="a" * 40, brief_file=str(brief),
                      brief_sha256=f.file_digest(brief), model="fixture",
                      reasoning_effort="high", task="fixture", worktree=str(self.root))
        allocation = copy.deepcopy(seed()[2]["bootstrap"])

        def accept(key, values, replace):
            allocation["inputs"] = {k: v for k, v in values.items() if k != "allocation"}
            self.c.put_allocation("promotion", allocation, replace,
                                  self.c.snapshot()["revision"], {"synthetic": True})
            self.c.event({"id": key})
            packet = self.c.begin_manager()
            action = dict(id=key, kind="repair", workstream="delivery", sprint="PF80",
                          rationale="fixture", inputs=values, timeout_seconds=60,
                          expected_revision=packet["state_revision"])
            self.c.accept_decision(packet["manager_run"],
                                   dict(state_revision=packet["state_revision"], actions=[action]),
                                   {"synthetic": True})

        accept("old", inputs, False)
        accept("new", freeze_worker_inputs(inputs, provider="fixture", policy="--yolo"), True)
        binary = base / "fake-binary"
        f.write_file(binary, b"fixture; never executed")
        runs = base / "runs"
        runs.mkdir(mode=0o700)
        self.transport = dict(kind="tmux", binary=str(binary), binary_sha256=f.file_digest(binary),
                              tmux=str(binary), runs_dir=str(runs), auth_link=str(base / "auth.json"))
        self.transport_path = base / "transport.json"
        f.write_json(self.transport_path, self.transport)
        evidence = base / "fixture-evidence.md"
        f.write_file(evidence, b"SYNTHETIC declaration for tests only. No real inference or approvals.")
        self.manifest = fixture_manifest(evidence, self.c.snapshot(), "old", "new", self.transport)
        self.calls.clear()

    def evaluate(self):
        return gate.evaluate(self.manifest, self.args.root, self.transport_path, ["new"])

    def test_positive_readonly_gate(self):
        self.prepare()
        before = {str(p): p.read_bytes() for p in Path(self.tmp.name).rglob("*") if p.is_file()}
        result = self.evaluate()
        self.assertTrue(result["ok"], result)
        self.assertEqual(3, result["plan"]["decision"]["generation"])
        self.assertEqual([], self.calls)
        self.assertEqual(before, {str(p): p.read_bytes()
                                 for p in Path(self.tmp.name).rglob("*") if p.is_file()})

    def test_each_of_five_items_names_unmet_condition(self):
        self.prepare()
        original = copy.deepcopy(self.manifest)
        mutations = (
            (1, lambda m: m["preparation"].pop("adoption")),
            (2, lambda m: m["replacements"]["new"].update(old_action="new")),
            (3, lambda m: f.write_file(self.root / "foreign.pending", b"preserve")),
            (4, lambda m: m["qualification"].update(real_ack=0)),
            (5, lambda m: m["audit"].update(dispatchers_quiescent=False)),
        )
        for item, mutate in mutations:
            with self.subTest(item=item):
                self.manifest = copy.deepcopy(original)
                mutate(self.manifest)
                result = self.evaluate()
                self.assertFalse(result["ok"])
                self.assertEqual("UNMET", result["results"][item - 1]["status"])
                self.assertEqual(gate.ITEMS[item], result["results"][item - 1]["name"])
                if item == 3:
                    (self.root / "foreign.pending").unlink()  # Test-owned fixture teardown only.

    def test_unapplied_operation_refuses_before_disarm(self):
        self.prepare()
        self.sql("INSERT INTO operations(op_id,action_id,effect,phase) VALUES('x','x','launch','intent')")
        result = self.evaluate()
        self.assertFalse(result["ok"])
        self.assertIn("settled_operations", result["results"][4]["reason"])
        self.assertEqual("armed", owner.activation_status(self.config_path)["state"])
        self.assertEqual([], self.calls)

    def test_existing_schedule_hold_is_not_silently_recovered(self):
        self.prepare()
        tick_path = self.args.root / "tick.json"
        tick = owner.load(tick_path)
        tick["hold"] = "interrupted_tick"
        f.write_json(tick_path, tick)
        result = self.evaluate()
        self.assertFalse(result["ok"])
        self.assertEqual("schedule_recovery_required", result["results"][4]["reason"])
        self.assertEqual("armed", owner.activation_status(self.config_path)["state"])

    def test_bad_revision_and_missing_authority_refuse(self):
        self.prepare()
        self.manifest["activation"]["revision"] = 0
        result = self.evaluate()
        self.assertIn("activation_revision_required", result["results"][4]["reason"])
        self.manifest["activation"]["revision"] = 1
        self.manifest.pop("activation_authority")
        self.assertIn("activation_authority_redacted_evidence_required",
                      self.evaluate()["results"][4]["reason"])

    def test_limited_route_requires_named_acceptance_and_stays_limited(self):
        self.prepare()
        q = self.manifest["qualification"]
        q.update(mode="limited", limitation="no real ACK yet")
        self.assertIn("named_product_authority_limitation_required",
                      self.evaluate()["results"][3]["reason"])
        # Fictional record exercises schema only; it is not a Travis approval.
        q.update(accepted_by="Travis Good", allowed_scope="synthetic first action",
                 remaining_proof="real isolated cycle", acceptance=q["evidence"])
        result = self.evaluate()
        self.assertTrue(result["ok"], result)
        self.assertTrue(result["results"][3]["detail"].startswith("LIMITED:"))

    def test_recipe_stale_publication_preserves_every_existing_file(self):
        self.prepare()
        pending = self.root / "owner-recurrence.json.pending"
        f.write_file(pending, b"interrupted publication evidence")
        manifest_path = Path(self.tmp.name) / "preflight.json"
        f.write_json(manifest_path, self.manifest)
        before = {str(p): p.read_bytes() for p in Path(self.tmp.name).rglob("*") if p.is_file()}
        recipe = Path(__file__).with_name("owner-handoff-80-promotion.md").read_text()
        code = recipe.split("-B - <<'PY'\n", 1)[1].split("\nPY\n", 1)[0]
        env = dict(OWNER94_PREFLIGHT=str(manifest_path), OWNER80_SCHEDULE=str(self.args.root),
                   OWNER80_TRANSPORT=str(self.transport_path), OWNER80_ACTIONS="new",
                   OWNER80_DECISION_ID="synthetic-promotion", OWNER80_DECISION_REVISION="1",
                   OWNER80_AUTHORITY="disposable rehearsal only")
        output = io.StringIO()
        with patch.dict(os.environ, env), contextlib.redirect_stdout(output), \
                patch.object(owner, "disarm_owner") as disarm, \
                patch.object(activate, "owner_activation") as uninstall:
            with self.assertRaises(SystemExit) as refusal:
                exec(compile(code, "exact-promotion-recipe", "exec"), {})
        self.assertEqual(2, refusal.exception.code)
        disarm.assert_not_called()
        uninstall.assert_not_called()
        self.assertEqual([], self.calls)
        self.assertIn("stale_publication_pending", output.getvalue())
        self.assertEqual(before, {str(p): p.read_bytes()
                                 for p in Path(self.tmp.name).rglob("*") if p.is_file()})
        self.assertEqual("armed", owner.activation_status(self.config_path)["state"])
        self.assertEqual("installed", owner.load(self.args.root / "installation.json")["phase"])
        print(json.dumps(dict(rehearsal="stale_publication", recipe_exit=2, disarms=0,
                              uninstalls=0, launchctl_mutations=0, files_unchanged=len(before),
                              admission="armed", installation="installed",
                              pending_preserved=True, real_inference=False)))

    def test_cli_reports_nonzero_with_all_item_names(self):
        self.prepare()
        manifest_path = Path(self.tmp.name) / "missing.json"
        f.write_json(manifest_path, {})
        env = dict(PATH=f.SAFE_PATH, PYTHONPATH=str(Path(owner.__file__).parent),
                   CORBANU_TEST_NO_NATIVE_KEYRING="1",
                   **{k: self.tmp.name for k in ("HOME", "CODEX_HOME", "CORBANU_HOME", "PFTERMINAL_HOME")})
        result = subprocess.run([sys.executable, "-B", str(Path(gate.__file__)),
                                 "--manifest", str(manifest_path), "--schedule", str(self.args.root),
                                 "--transport", str(self.transport_path), "--actions", "new"],
                                env=env, text=True, capture_output=True, timeout=10)
        self.assertEqual(2, result.returncode)
        report = json.loads(result.stdout)
        self.assertEqual(list(gate.ITEMS.values()), [r["name"] for r in report["results"]])
        self.assertEqual("UNMET", report["results"][0]["status"])


if __name__ == "__main__":
    unittest.main(verbosity=2)

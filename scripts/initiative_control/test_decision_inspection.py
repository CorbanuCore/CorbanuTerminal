"""Engineering regressions; frozen DEC021 functional acceptance stays separate."""
import copy
import json
import os
import unittest
from unittest.mock import patch

import activate
import attention
import control
import decision_alerts as alerts
import decision_feed as transport
import decision_replies as replies
import decisions as d
import export
import test_decision_feed as existing
import test_decision_alerts as alert_tests
import test_decision_replies as reply_tests
from test_decision_alerts import OWNER, PIN, REMOTE
from test_decisions import NOW, LATER, revision

CANARY = "SYNTHETIC_SECRET inspection-canary RAW_PRIVATE_LOG"


class InspectionTests(unittest.TestCase):
    save = existing.FeedTests.save
    bundle = existing.FeedTests.bundle
    activate = existing.FeedTests.activate
    publish = existing.FeedTests.publish

    def setUp(self):
        existing.FeedTests.setUp(self)
        self.save(self.value)
        self.value = revision(self.value, "resolved")
        sibling = copy.deepcopy(self.value["decisions"][0])
        sibling["id"] = "withheld-id"
        for record in sibling["revisions"]:
            record["summary"] = "Withheld summary"
            record["evidence"][0]["path"] = "qa/withheld-path.md"
        self.value["decisions"].append(sibling)
        self.assertEqual(d.validate(self.value, NOW), self.value)
        self.save(self.value)
        self.path = self.state / "decisions.fixture.json"
        self.baseline = self.path.read_bytes()
        self.assertEqual(d.validate(self.baseline, NOW), self.value)
        (self.root / "validated-baseline.json").write_bytes(self.baseline)

    def poison(self, value=None):
        value = copy.deepcopy(self.value) if value is None else value
        if value == self.value:
            value["decisions"][1]["revisions"][-1]["question"] = CANARY
        control.atomic_json(self.path, value)
        return self.path.read_bytes()

    def captured(self):
        raw, pin = transport.capture(self.state, NOW)
        snapshot = json.loads(raw)
        self.assertIsNone(snapshot["feed"])
        self.assertEqual(snapshot["status"], "invalid")
        health = transport.health(snapshot, {"decision_feed": pin}, NOW)
        self.assertEqual(health["state"], "unknown")
        self.assertIsNone(health["open_count"])
        return snapshot, raw, pin

    def test_actual_export_activate_publish_retains_baseline_history_and_links(self):
        baseline, _ = self.bundle()
        baseline_bytes = (baseline / "source" / transport.ARTIFACT).read_bytes()
        self.activate(baseline)
        _, page, _ = self.publish()
        self.assertIn("Withheld summary", page)
        transport.project_slack(self.state, None, NOW)
        poisoned = self.poison()
        target, manifest = self.bundle()
        frozen = (target / "source" / transport.ARTIFACT).read_bytes()
        snapshot = transport.read_snapshot(target / "source", manifest, NOW)
        self.assertEqual(snapshot["schema"], 3)
        self.assertEqual(snapshot["withheld_count"], 1)
        self.assertEqual(snapshot["inspection"], {**self.value, "decisions": self.value["decisions"][:1]})
        self.assertIsNone(manifest["decision_feed"]["payload_digest"])
        self.assertNotIn("slack", snapshot)
        self.activate(target)
        for at in (NOW, LATER, LATER):
            data, page, health = self.publish(at)
            self.assertEqual(data["decision_snapshot"], snapshot)
            fragment = page.split('<section id="decisions"', 1)[1].split('</section>', 1)[0]
            for text in ("incomplete input", "Inspection only", "Total open count unknown", "Retained revision 1", "Sanitized test plan", NOW):
                self.assertIn(text, fragment)
            for path in (existing.SPRINT, attention.HISTORY):
                self.assertIn('href="' + control.route(path) + '"', fragment)
                self.assertTrue((self.install / "site/current" / control.route(path)).is_file())
            for text in ("Fresh manager assessment", "No open decisions found", "Open decisions:", "Slack observation:", "Answer in the manager task"):
                self.assertNotIn(text, fragment)
            self.assertEqual(health["decision_feed"]["state"], "unknown")
            self.assertIsNone(health["decision_feed"]["open_count"])
            self.assertEqual(health["decision_feed"]["slack"]["state"], "unknown")
            for text in (CANARY, "inspection-canary", "withheld-id", "withheld-path", "Withheld summary"):
                self.assertNotIn(text, fragment + frozen.decode() + json.dumps(manifest) + json.dumps(health))
        self.assertEqual(self.path.read_bytes(), poisoned)
        self.assertEqual((target / "source" / transport.ARTIFACT).read_bytes(), frozen)
        self.assertEqual((baseline / "source" / transport.ARTIFACT).read_bytes(), baseline_bytes)
        self.assertEqual((self.root / "validated-baseline.json").read_bytes(), self.baseline)

    def test_secrets_in_current_history_resolution_and_links_withhold_whole_record(self):
        for index, field in ((0, "background"), (1, "question"), (0, "path"), (1, "label"), (1, "answer")):
            with self.subTest(index=index, field=field):
                value = copy.deepcopy(self.value)
                record = value["decisions"][1]["revisions"][index]
                target = record["evidence"][0] if field in ("path", "label") else record["resolution"] if field == "answer" else record
                target[field] = CANARY
                before = self.poison(value)
                snapshot, raw, pin = self.captured()
                self.assertEqual(snapshot["inspection"]["decisions"], self.value["decisions"][:1])
                self.assertNotIn("inspection-canary", raw.decode() + json.dumps(pin) + transport.render(snapshot, NOW, [], {}))
                with self.assertRaises(d.Invalid):
                    d.save_fixture(self.state, value, d.digest(self.value), NOW)
                self.assertEqual(self.path.read_bytes(), before)

    def test_duplicate_ids_all_copies_and_invalid_histories_are_withheld(self):
        for malformed in (False, True):
            value = copy.deepcopy(self.value)
            duplicate = copy.deepcopy(value["decisions"][1])
            if malformed:
                duplicate = {"id": duplicate["id"], "revisions": None}
            value["decisions"].insert(0, duplicate)
            self.poison(value)
            snapshot, _, _ = self.captured()
            self.assertEqual(snapshot["withheld_count"], 2)
            self.assertEqual(snapshot["inspection"]["decisions"], self.value["decisions"][:1])
        for change in ("gap", "missing", "reordered", "wrong-answer"):
            value = copy.deepcopy(self.value)
            history = value["decisions"][1]["revisions"]
            if change == "gap":
                history[-1]["revision"] = 3
            elif change == "missing":
                history.pop(0)
            elif change == "reordered":
                history.reverse()
            else:
                history[-1]["resolution"]["answered_revision"] = 2
            self.poison(value)
            self.assertEqual(self.captured()[0]["inspection"]["decisions"], self.value["decisions"][:1])

    def test_open_sibling_stays_inspection_only_at_each_refresh(self):
        value = revision(self.value)
        self.save(value)
        self.assertEqual(d.load_fixture(self.state, NOW), value)
        value["decisions"][1]["revisions"][0]["question"] = CANARY
        self.poison(value)
        snapshot, _, _ = self.captured()
        for at in (NOW, LATER):
            page = transport.render(snapshot, at, [], {})
            self.assertIn("Inspection records (recorded open)", page)
            self.assertIn('id="decision-choice-1"', page)
            self.assertIn("Total open count unknown", page)
            self.assertNotIn("Open decisions:", page)
            self.assertNotIn("Fresh manager assessment", page)
            self.assertNotIn('id="oldest-open-decision-age"', page)

    def test_invalid_envelopes_ambiguous_json_oversize_and_no_survivors(self):
        raw = d.canonical(self.value)
        bad = [raw + b" " * d.MAX_BYTES, raw.replace(b'"schema":1', b'"schema":1,"schema":1'),
               raw.replace(b'"owner":"Travis"', b'"owner":"Travis","owner":"Travis"', 1),
               raw.replace(b'"question":"Five or ten testers?"', b'"question":NaN', 1), b'[]', b'{']
        bad += [d.canonical({**self.value, key: value}) for key, value in
                (("schema", True), ("feed_id", CANARY), ("revision", 0), ("assessed_at", "2099-01-01T00:00:00Z"), ("extra", CANARY), ("decisions", {}))]
        all_bad = copy.deepcopy(self.value)
        for item in all_bad["decisions"]:
            item["revisions"][0]["question"] = CANARY
        bad += [d.canonical(all_bad), d.canonical({**self.value, "decisions": self.value["decisions"] * 2})]
        for payload in bad:
            self.path.write_bytes(payload)
            snapshot, output, _ = self.captured()
            self.assertNotIn("inspection", snapshot)
            self.assertNotIn("choice-1", output.decode())
            self.assertNotIn("inspection-canary", output.decode())

    def test_untrusted_files_never_expose_siblings(self):
        raw = self.poison()
        for kind in ("file-mode", "root-mode", "owner", "hardlink", "symlink", "fifo", "directory"):
            with self.subTest(kind=kind):
                if self.path.exists():
                    self.path.unlink()
                self.path.write_bytes(raw)
                self.path.chmod(0o600)
                if kind == "file-mode":
                    self.path.chmod(0o640)
                elif kind == "root-mode":
                    self.state.chmod(0o750)
                elif kind == "hardlink":
                    os.link(self.path, self.root / "second-link")
                elif kind in ("symlink", "fifo", "directory"):
                    self.path.unlink()
                    if kind == "symlink":
                        self.path.symlink_to(self.root / "validated-baseline.json")
                    elif kind == "fifo":
                        os.mkfifo(self.path)
                    else:
                        self.path.mkdir()
                with patch.object(d.os, "getuid", return_value=os.getuid() + 1) if kind == "owner" else patch.object(d.os, "getuid", return_value=os.getuid()):
                    self.assertNotIn("inspection", self.captured()[0])
                self.state.chmod(0o700)
                if kind == "directory":
                    self.path.rmdir()

    def test_partial_transfer_tamper_drift_and_last_good(self):
        self.poison()
        target, manifest = self.bundle()
        self.activate(target)
        self.publish()
        last_good = (self.install / "site/current").resolve()
        path = target / "source" / transport.ARTIFACT
        original = path.read_bytes()
        snapshot = json.loads(original)
        for field, value in (("withheld_count", 2), ("inspection", {**snapshot["inspection"], "feed_id": "changed"})):
            path.write_bytes(d.canonical({**snapshot, field: value}))
            with patch.object(activate.subprocess, "run") as services, self.assertRaisesRegex(ValueError, "transfer"):
                activate.activate(self.install, target, self.root / "units")
            services.assert_not_called()
        for changes in ({"withheld_count": True}, {"withheld_count": 0}, {"withheld_count": d.MAX_BYTES + 1},
                        {"feed": snapshot["inspection"]}, {"status": "valid"}, {"slack": {}},
                        {"inspection": {**snapshot["inspection"], "decisions": []}}, {"schema": 2}):
            payload = d.canonical({**snapshot, **changes})
            path.write_bytes(payload)
            altered = copy.deepcopy(manifest)
            altered["decision_feed"]["digest"] = transport.sha(payload)
            with self.assertRaisesRegex(ValueError, "transfer"):
                transport.read_snapshot(target / "source", altered, NOW)
        path.write_bytes(original)
        self.assertEqual((self.install / "site/current").resolve(), last_good)
        self.assertEqual(transport.read_snapshot(target / "source", manifest, LATER), snapshot)
        self.path.write_bytes(self.path.read_bytes().replace(b"inspection-canary", b"another-canary"))
        with self.assertRaisesRegex(ValueError, "Decision input changed"):
            transport.verify_input(self.state, manifest, NOW)
        previous = (self.state / "source.json").read_bytes()
        verify = export.verify_source
        def drift(*args):
            verify(*args)
            self.path.write_bytes(self.path.read_bytes() + b" ")
        with patch.object(export, "verify_source", side_effect=drift), self.assertRaisesRegex(ValueError, "Decision input changed"):
            self.bundle()
        self.assertEqual((self.state / "source.json").read_bytes(), previous)


class StrictSlackTests(alert_tests.SlackFixture):
    intake = reply_tests.ReplyTests.intake
    manager = reply_tests.ReplyTests.manager
    interpret = reply_tests.ReplyTests.interpret
    queue = reply_tests.ReplyTests.queue

    def setUp(self):
        super().setUp()
        self.send()

    def test_partial_input_cannot_resolve_or_dispatch_existing_reply(self):
        key = self.queue()
        baseline = d.load_fixture(self.feed_root, NOW)
        self.assertEqual(d.validate(baseline, NOW), baseline)
        bad = copy.deepcopy(baseline)
        sibling = copy.deepcopy(bad["decisions"][0])
        sibling["id"] = "withheld-id"
        sibling["revisions"][0]["question"] = CANARY
        bad["decisions"].append(sibling)
        path = self.feed_root / "decisions.fixture.json"
        control.atomic_json(path, bad)
        before = path.read_bytes()
        snapshot = json.loads(transport.capture(self.feed_root, NOW)[0])
        self.assertEqual(snapshot["schema"], 3)
        ledger = self.store.read("replies")
        operations = [lambda: alerts.prepare(bad, "choice-1", REMOTE, PIN, OWNER, NOW),
                      lambda: alerts.prepare(snapshot, "choice-1", REMOTE, PIN, OWNER, NOW),
                      lambda: transport.project_slack(self.feed_root, None, NOW), self.interpret,
                      lambda: replies.dispatch(self.store, self.feed_root, key, OWNER, lambda _: self.fail("dispatch"), NOW)]
        for operation in operations:
            with self.assertRaises(d.Invalid):
                operation()
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(self.store.read("replies"), ledger)

    def test_recorded_reply_cannot_persist_resolution_into_partial_input(self):
        self.intake()
        key = self.interpret()
        baseline = d.load_fixture(self.feed_root, NOW)
        self.assertEqual(d.validate(baseline, NOW), baseline)
        sibling = copy.deepcopy(baseline["decisions"][0])
        sibling["id"] = "withheld-id"
        sibling["revisions"][0]["question"] = CANARY
        path = self.feed_root / "decisions.fixture.json"
        control.atomic_json(path, {**baseline, "decisions": baseline["decisions"] + [sibling]})
        before, ledger = path.read_bytes(), self.store.read("replies")
        self.assertEqual(json.loads(transport.capture(self.feed_root, NOW)[0])["schema"], 3)
        with self.assertRaises(d.Invalid):
            replies.resume(self.store, self.feed_root, key, OWNER, NOW)
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(self.store.read("replies"), ledger)

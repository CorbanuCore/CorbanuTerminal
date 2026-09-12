import json
import copy
from html.parser import HTMLParser
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import activate
import attention
import control
import decision_feed as transport
import decisions as d
import export
from test_decisions import NOW, LATER, fixture, revision
from test_control import run

REPO = control.HERE.parents[1]
SPRINT = "docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md"


class FeedTests(unittest.TestCase):
    def test_slack_off_projection_cli_and_registered_off_without_sdk(self):
        self.save(self.value)
        original = (self.state / "decisions.fixture.json").read_bytes()
        env = dict(os.environ, PYTHONPATH=str(control.HERE) + os.pathsep + os.environ.get("PYTHONPATH", ""))
        missing = self.root / "must-not-be-created"
        command = [sys.executable, "-B", "-c", "import sys; sys.modules['slack_sdk']=None; import control; sys.argv=['control.py']+sys.argv[1:]; control.main()"]
        status = subprocess.run(command + ["decision-slack", "status", "--store", str(missing)], env=env, capture_output=True, text=True)
        self.assertEqual(status.returncode, 0, status.stderr)
        self.assertEqual(json.loads(status.stdout)["state"], "off")
        projected = subprocess.run(command + ["decision-slack", "--publish-state", str(self.state), "project-status", "--store", str(missing)], env=env, capture_output=True, text=True)
        self.assertEqual(projected.returncode, 0, projected.stderr)
        value = json.loads(projected.stdout)
        self.assertEqual(value["status"]["state"], "off")
        self.assertEqual(value["feed_digest"], d.digest(self.value))
        self.assertEqual((self.state / "decisions.fixture.json").read_bytes(), original)
        self.assertFalse(missing.exists())
        self.assertEqual((self.state / transport.SLACK_FILE).stat().st_mode & 0o077, 0)
        denied = subprocess.run(command + ["decision-slack", "--publish-state", str(self.state), "send", "--store", str(missing)], env=env, capture_output=True, text=True)
        self.assertNotEqual(denied.returncode, 0)

    def test_slack_v2_roundtrip_fresh_stale_rollback_and_input_drift(self):
        self.save(self.value)
        cache = transport.project_slack(self.state, None, NOW)
        original = (self.state / "decisions.fixture.json").read_bytes()
        target, pin = self.bundle()
        self.assertEqual(pin["decision_feed"]["schema"], 2)
        self.activate(target)
        _, page, health = self.publish()
        self.assertIn("Slack observation: off", page)
        self.assertIn("question revision 1", page)
        self.assertEqual(health["decision_feed"]["slack"]["state"], "off")
        self.assertNotIn(transport.SLACK_FILE, pin["files"])
        before = (target / "source" / transport.ARTIFACT).read_bytes()
        _, later, old_health = self.publish(LATER)
        self.assertIn("Slack observation: stale", later)
        self.assertEqual(old_health["decision_feed"]["slack"]["state"], "stale")
        self.assertEqual((target / "source" / transport.ARTIFACT).read_bytes(), before)
        changed = copy.deepcopy(cache)
        changed["assessed_at"] = LATER
        control.atomic_json(self.state / transport.SLACK_FILE, changed)
        with self.assertRaisesRegex(ValueError, "Decision input changed"):
            transport.verify_input(self.state, pin, LATER)
        self.assertEqual((self.state / "decisions.fixture.json").read_bytes(), original)
        newer, _ = self.bundle(LATER)
        self.activate(newer, LATER)
        self.publish(LATER)
        self.activate(target, LATER)
        _, _, rolled = self.publish(LATER)
        self.assertEqual(rolled["decision_feed"]["slack"]["assessed_at"], NOW)

    def test_bad_slack_input_never_discards_valid_decision_context(self):
        self.save(self.value)
        original = transport.project_slack(self.state, None, NOW)
        path = self.state / transport.SLACK_FILE
        bad = []
        for field, value in (("feed_digest", "0" * 64), ("schema", True), ("assessed_at", "2099-01-01T00:00:00Z"), ("extra", "SYNTHETIC_SECRET no-export")):
            bad.append({**original, field: value})
        altered = copy.deepcopy(original)
        altered["decisions"][0]["id"] = "different-question"
        bad += [altered, {**original, "decisions": original["decisions"] * 2}]
        for value in bad:
            control.atomic_json(path, value)
            raw, pin = transport.capture(self.state, NOW)
            snapshot = json.loads(raw)
            self.assertEqual(snapshot["feed"], self.value)
            self.assertEqual(snapshot["slack_status"], "invalid")
            page = transport.render(snapshot, NOW, [], {})
            self.assertIn(self.value["decisions"][0]["revisions"][0]["question"], page)
            self.assertIn("Slack status unavailable", page)
            self.assertNotIn("no-export", raw.decode() + json.dumps(pin) + page)
        for raw in (b'{"schema":1,"schema":1}', b"x" * (transport.SLACK_LIMIT + 1)):
            path.write_bytes(raw)
            self.assertEqual(json.loads(transport.capture(self.state, NOW)[0])["slack_status"], "invalid")
        path.unlink()
        path.symlink_to(self.root / "missing")
        self.assertEqual(json.loads(transport.capture(self.state, NOW)[0])["slack_status"], "invalid")
        path.unlink()
        os.mkfifo(path)
        self.assertEqual(json.loads(transport.capture(self.state, NOW)[0])["slack_status"], "invalid")
        path.unlink()
        self.assertEqual(json.loads(transport.capture(self.state, NOW)[0])["schema"], 1)

    def test_status_context_tamper_in_pinned_transfer_is_rejected(self):
        self.save(self.value)
        transport.project_slack(self.state, None, NOW)
        target, pin = self.bundle()
        path = target / "source" / transport.ARTIFACT
        value = json.loads(path.read_bytes())
        value["slack"]["decisions"][0]["context_digest"] = "0" * 64
        path.write_bytes(d.canonical(value))
        altered = copy.deepcopy(pin)
        altered["decision_feed"]["digest"] = transport.sha(path.read_bytes())
        with self.assertRaisesRegex(ValueError, "Decision feed transfer"):
            transport.read_snapshot(target / "source", altered, NOW)

    def test_registered_slack_help_needs_no_transport_initialization(self):
        result = subprocess.run([sys.executable, "-B", str(control.HERE / "control.py"),
                                 "decision-slack", "--help"], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("decision-slack", result.stdout)
        self.assertNotIn("Traceback", result.stderr)

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve())
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.state = self.root / "manager"
        control.atomic_json(self.state / "control.json", {"human_tests": [], "tasknode": {"enabled": False}})
        self.install = self.root / "corbanu-control"
        self.number = 0
        self.value = fixture()
        record = self.value["decisions"][0]["revisions"][0]
        record["sprints"][0]["path"] = SPRINT
        record["sprints"].append(dict(sprint_id="PF-76-S01", path=attention.HISTORY, historical=True))
        record["summary"] += " / PF-76-S01 historical context"
        record["evidence"] += [dict(label="Unpublished synthetic context", path="qa/not-published.md", assessed_at=None)]
        record["evidence"][0]["path"] = SPRINT

    def save(self, value):
        old = d.load_fixture(self.state, LATER)
        return d.save_fixture(self.state, value, d.digest(old) if old else None, LATER)

    def bundle(self, at=NOW):
        self.number += 1
        target = self.install / "incoming" / ("upload-" + f"{self.number:032x}")
        with patch.object(export, "now", return_value=at):
            manifest = export.export(REPO, self.state, target)
        return target, manifest

    def activate(self, target, at=NOW):
        with patch.object(activate, "now", return_value=at), patch.object(activate.subprocess, "run") as services:
            activate.activate(self.install, target, self.root / "units")
        self.assertEqual(services.call_count, 5)

    def publish(self, at=NOW):
        with patch.object(control, "now", return_value=at):
            data = control.publish(self.install / "source", self.install / "state", self.install / "site")
        site = self.install / "site"
        return data, (site / "current/index.html").read_text(), json.loads((site / "health.json").read_text())

    def test_chain_history_links_notices_facilities_and_repeat_assessment(self):
        self.save(self.value)
        self.save(revision(self.value, "resolved"))
        control.report(self.state, {**run(), "sprint_id": "PF-76-S01"})
        target, manifest = self.bundle()
        snapshot_bytes = (target / "source" / transport.ARTIFACT).read_bytes()
        self.assertNotIn(transport.ARTIFACT, manifest["files"])
        self.assertEqual(manifest["decision_feed"]["digest"], transport.sha(snapshot_bytes))
        self.assertEqual(manifest["decision_feed"]["payload_digest"], d.digest(revision(self.value, "resolved")))
        transport.verify_input(self.state, manifest, NOW)
        self.activate(target)
        data, page, health = self.publish()
        self.assertEqual(data["decision_snapshot"]["feed"], revision(self.value, "resolved"))
        self.assertLess(page.index('id="decisions"'), page.index('id="initiatives"'))
        for text in ("Operations notices", "Slack not connected", "Answer in the manager task", "Live delivery disabled", "Retained revision 1", "against question revision 1", "unavailable context", 'id="decision-choice-1"'):
            self.assertIn(text, page)
        for path in (SPRINT, attention.HISTORY):
            self.assertIn('href="' + control.route(path) + '"', page)
            self.assertTrue((self.install / "site/current" / control.route(path)).is_file())
        for path in ("state/decisions.fixture.json", "qa/not-published.md"):
            self.assertFalse((target / path).exists())
        self.assertIn("7 registered interfaces", (self.install / "site/current/facilities.html").read_text())
        before = (self.state / "decisions.fixture.json").read_bytes()
        for _ in range(2):
            later_data, later_page, later_health = self.publish(LATER)
            self.assertEqual(later_data["decision_snapshot"], data["decision_snapshot"])
            self.assertEqual(later_health["decision_feed"]["assessed_at"], NOW)
            self.assertEqual(later_health["decision_feed"]["state"], "stale")
            self.assertIn("Stale: current decisions unknown", later_page)
            self.assertNotIn("No open decisions found", later_page)
        self.assertEqual((self.state / "decisions.fixture.json").read_bytes(), before)
        self.assertEqual((target / "source" / transport.ARTIFACT).read_bytes(), snapshot_bytes)
        self.assertEqual(health["decision_feed"]["state"], "fresh")

    def test_empty_missing_invalid_fresh_stale_and_new_assessment_chain(self):
        for status in ("missing", "invalid", "valid"):
            path = self.state / "decisions.fixture.json"
            if status == "invalid":
                control.atomic_json(path, {"raw_private_log": "SYNTHETIC_SECRET pf80-feed-leak-xyz"})
            if status == "valid":
                path.unlink()
                self.save({**self.value, "decisions": []})
            target, pin = self.bundle()
            self.activate(target)
            _, page, health = self.publish()
            self.assertEqual(health["decision_feed"]["input_status"], status)
            self.assertEqual(health["decision_feed"]["state"], "fresh" if status == "valid" else "unknown")
            self.assertEqual(health["decision_feed"]["open_count"], 0 if status == "valid" else None)
            self.assertEqual("No open decisions found" in page, status == "valid")
            self.assertNotIn("pf80-feed-leak-xyz", page + json.dumps(pin) + (target / "source" / transport.ARTIFACT).read_text())
            _, old_page, old_health = self.publish(LATER)
            self.assertEqual(old_health["decision_feed"]["state"], "stale" if status == "valid" else "unknown")
            self.assertNotIn("No open decisions found", old_page)
        self.save({**self.value, "revision": 2, "assessed_at": LATER, "decisions": []})
        target, _ = self.bundle(LATER)
        self.activate(target, LATER)
        _, page, health = self.publish(LATER)
        self.assertIn("No open decisions found", page)
        self.assertEqual(health["decision_feed"]["assessed_at"], LATER)

    def test_transfer_tamper_rejected_before_activation_and_lastgood_retained(self):
        self.save(self.value)
        target, manifest = self.bundle()
        self.activate(target)
        self.publish()
        previous = (self.install / "site/current").resolve()
        previous_page = (previous / "index.html").read_bytes()
        changed, _ = self.bundle()
        path = changed / "source" / transport.ARTIFACT
        path.write_bytes(path.read_bytes() + b" ")
        with patch.object(activate.subprocess, "run", side_effect=AssertionError("no service calls")), self.assertRaisesRegex(ValueError, "Decision feed transfer"):
            activate.activate(self.install, changed, self.root / "units")
        self.assertEqual((self.install / "source").resolve(), target / "source")
        self.assertEqual(json.loads((self.install / "state/source.json").read_text()), manifest)
        (target / "source" / transport.ARTIFACT).unlink()
        with self.assertRaisesRegex(ValueError, "Decision feed transfer"):
            self.publish(LATER)
        self.assertEqual((self.install / "site/current").resolve(), previous)
        self.assertEqual((previous / "index.html").read_bytes(), previous_page)
        self.assertFalse(json.loads((self.install / "site/health.json").read_text())["ok"])
        retained = json.loads((self.install / "site/health.json").read_text())["decision_feed"]
        self.assertEqual((retained["assessed_at"], retained["state"]), (NOW, "stale"))

    def test_strict_snapshot_shape_digest_identity_paths_and_safe_read_errors(self):
        self.save(self.value)
        raw, pin = transport.capture(self.state, NOW)
        repo = self.root / "source"
        repo.mkdir()
        path = repo / transport.ARTIFACT
        path.write_bytes(raw)
        for key, value in (("artifact", "../manager/decisions.fixture.json"), ("schema", True), ("revision", True), ("digest", "a" * 64), ("payload_digest", "b" * 64), ("assessed_at", LATER), ("feed_id", "wrong"), ("extra", "canary")):
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, "Decision feed transfer"):
                transport.read_snapshot(repo, {"decision_feed": {**pin, key: value}}, NOW)
        for payload in (b"{" * 2000, b"x" * (transport.LIMIT + 1), b'{"schema":1,"schema":1}', raw + b" ", d.canonical({"schema": 1, "status": "missing", "feed": self.value})):
            path.write_bytes(payload)
            with self.assertRaisesRegex(ValueError, "Decision feed transfer"):
                transport.read_snapshot(repo, {"decision_feed": {**pin, "digest": transport.sha(payload)}}, NOW)
        path.unlink()
        path.symlink_to(self.state / "decisions.fixture.json")
        with self.assertRaisesRegex(ValueError, "Decision feed transfer"):
            transport.read_snapshot(repo, {"decision_feed": pin}, NOW)
        path.unlink()
        os.mkfifo(path)
        with self.assertRaisesRegex(ValueError, "Decision feed transfer"):
            transport.read_snapshot(repo, {"decision_feed": pin}, NOW)

    def test_invalid_private_input_is_safe_unknown_and_drift_is_detected(self):
        path = self.state / "decisions.fixture.json"
        for payload in (b'{"secret":"SYNTHETIC_SECRET canary"}', b'{"schema":1,"schema":1}', b"x" * (d.MAX_BYTES + 1)):
            path.write_bytes(payload)
            path.chmod(0o600)
            raw, pin = transport.capture(self.state, NOW)
            self.assertEqual(json.loads(raw), dict(schema=1, status="invalid", feed=None))
            self.assertNotIn("canary", raw.decode() + json.dumps(pin))
            transport.verify_input(self.state, {"decision_feed": pin}, NOW)
            path.write_bytes(payload + b"changed")
            with self.assertRaisesRegex(ValueError, "Decision input changed"):
                transport.verify_input(self.state, {"decision_feed": pin}, NOW)
        path.unlink()
        path.symlink_to(self.root / "absent")
        self.assertEqual(transport.capture(self.state, NOW)[1]["status"], "invalid")
        path.unlink()
        self.save(self.value)
        path.chmod(0o644)
        self.assertEqual(transport.capture(self.state, NOW)[1]["status"], "invalid")

    def test_export_race_and_existing_verify_cli_detect_local_changes(self):
        self.save(self.value)
        target, manifest = self.bundle()
        command = [sys.executable, "-B", str(control.HERE / "export.py"), "--repo", str(REPO), "--state", str(self.state), "--verify", str(target / "state/source.json")]
        self.assertEqual(subprocess.run(command, capture_output=True).returncode, 0)
        self.save(revision(self.value))
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Decision input changed", result.stderr)
        before = (self.state / "source.json").read_bytes()
        original = export.verify_source
        def racing(*args):
            original(*args)
            self.save(revision(revision(self.value)))
        with patch.object(export, "verify_source", side_effect=racing), self.assertRaisesRegex(ValueError, "Decision input changed"):
            self.bundle()
        self.assertEqual((self.state / "source.json").read_bytes(), before)

    def test_transport_rejects_secret_canaries_in_history_context_and_destinations(self):
        for field in ("background", "path", "label", "answer"):
            value = revision(self.value, "resolved")
            record = value["decisions"][0]["revisions"][0]
            if field in ("path", "label"):
                record = record["evidence"][0]
            elif field == "answer":
                record = value["decisions"][0]["revisions"][-1]["resolution"]
            record[field] = "SYNTHETIC_SECRET pf80-feed-leak-xyz RAW_PRIVATE_LOG"
            control.atomic_json(self.state / "decisions.fixture.json", value)
            target, pin = self.bundle()
            snapshot = transport.read_snapshot(target / "source", pin, NOW)
            self.assertEqual(snapshot["status"], "invalid")
            rendered = transport.render(snapshot, NOW, [], {})
            self.assertIn("open count unknown", rendered)
            self.assertNotIn("pf80-feed-leak-xyz", rendered + (target / "source" / transport.ARTIFACT).read_text() + json.dumps(pin))

    def test_immutable_generations_rollback_and_legacy_unknown(self):
        self.save(self.value)
        original, _ = self.bundle()
        self.save({**revision(self.value, "resolved"), "assessed_at": LATER})
        newer, _ = self.bundle(LATER)
        self.activate(newer, LATER)
        self.publish(LATER)
        self.activate(original, LATER)  # Source rollback restores its own old assessment.
        data, page, health = self.publish(LATER)
        self.assertEqual(data["decision_snapshot"]["feed"], self.value)
        self.assertEqual(health["decision_feed"]["state"], "stale")
        self.assertIn("Last-known open: 1", page)
        manifest_path = original / "state/source.json"
        legacy = json.loads(manifest_path.read_text())
        del legacy["decision_feed"]
        control.atomic_json(manifest_path, legacy)
        self.activate(original, LATER)
        _, page, health = self.publish(LATER)
        self.assertIn("open count unknown", page)
        self.assertIsNone(health["decision_feed"]["assessed_at"])
        self.assertEqual(health["decision_feed"]["input_status"], "unrecorded")

    def test_exported_entrypoint_publishes_matching_health_and_manifest(self):
        self.value = json.loads(json.dumps(self.value).replace("2026-09-12T", "2026-09-11T"))
        self.save(self.value)
        target, pin = self.bundle()
        result = subprocess.run([sys.executable, "-B", str(target / "source/scripts/initiative_control/control.py"), "publish", "--repo", str(target / "source"), "--state", str(target / "state"), "--output", str(self.root / "preview")], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        manifest = json.loads((self.root / "preview/current/manifest.json").read_text())
        health = json.loads((self.root / "preview/health.json").read_text())
        self.assertEqual(manifest["decision_feed"], health["decision_feed"])
        self.assertEqual(manifest["source"], pin)
        self.assertEqual(health["decision_feed"]["assessed_at"], "2026-09-11T12:00:00Z")

    def test_destinationless_export_preserves_local_publication_as_unknown(self):
        for with_feed in (False, True):
            with self.subTest(with_feed=with_feed):
                if with_feed:
                    self.save(self.value)
                with patch.object(export, "now", return_value=NOW):
                    manifest = export.export(REPO, self.state, None)
                # No immutable source generation exists, so do not promise a feed.
                with patch.object(control, "now", return_value=NOW):
                    control.publish(REPO, self.state, self.root / "local-preview")
                health = json.loads((self.root / "local-preview/health.json").read_text())
                self.assertNotIn("decision_feed", manifest)
                self.assertEqual(health["decision_feed"]["state"], "unknown")
                self.assertIsNone(health["decision_feed"]["open_count"])
                self.assertFalse((REPO / transport.ARTIFACT).exists())

    def test_page_age_and_saved_link_javascript_without_browser_or_network(self):
        # Minimal DOM harness executes the shipped JS. Actual browser proof stays open.
        script = r'''
const assert = require("node:assert/strict"), vm = require("node:vm");
const source = require("node:fs").readFileSync(process.argv[1], "utf8");
for (const connected of [true, false]) {
  let now = Date.parse("2026-09-12T12:20:00Z"), timer;
  const texts = ["Source manager-fixture / revision 1; assessed 2026-09-12T12:00:00Z. Fresh manager assessment.", "Open decisions: 0. No open decisions found in this fresh assessment.", "Revision 1: open; raised 2026-09-12T12:00:00Z; context updated 2026-09-12T12:00:00Z. Context within freshness window.", "Evidence within freshness window; assessed 2026-09-12T12:00:00Z", "Unknown assessment time", "Question Fresh manager assessment."];
  const paragraphs = texts.map(text => ({textContent:text}));
  paragraphs[1].firstChild = paragraphs[1]; // This fixture has only a count text node.
  const section = {dataset:{assessedAt:"2026-09-12T12:00:00Z", freshSeconds:"1200"}, querySelectorAll:()=>paragraphs};
  paragraphs[0].parentElement = paragraphs[1].parentElement = section;
  paragraphs[3].firstElementChild = {tagName:"A", textContent:"Evidence within freshness window"};
  paragraphs[3].lastChild = {textContent:texts[3]};
  paragraphs[5].firstElementChild = {tagName:"STRONG"};
  const detail = {tagName:"DETAILS", parentElement:null, open:false, closest:()=>section};
  const freshness = {}, elements = {decisions:section, "decision-choice-1":detail, freshness};
  const document = {body:{dataset:{generation:"same",collected:"2026-09-12T12:00:00Z"}}, getElementById:id=>elements[id], querySelectorAll:()=>[]};
  const context = vm.createContext({document, Date:{now:()=>now,parse:Date.parse}, location:{hash:"#decision-choice-1"}, window:{addEventListener:()=>{}}, setInterval:f=>timer=f, fetch:async()=>{if (!connected) throw Error("offline"); return {ok:true,json:async()=>({ok:false})};}});
  vm.runInContext(source,context);
  assert.equal(detail.open,true);
  assert.match(paragraphs[0].textContent,/Fresh manager/);
  now += 1000; timer();
  assert.match(paragraphs[0].textContent,/Stale: current decisions unknown/);
  assert.match(paragraphs[0].textContent,/assessed 2026-09-12T12:00:00Z/);
  assert.equal(paragraphs[1].textContent,"Last-known open: 0.");
  assert.match(paragraphs[2].textContent,/Stale context/);
  assert.match(paragraphs[3].lastChild.textContent,/Stale evidence/);
  assert.equal(paragraphs[3].firstElementChild.textContent,"Evidence within freshness window");
  assert.equal(paragraphs[4].textContent,"Unknown assessment time");
  assert.equal(paragraphs[5].textContent,texts[5]);
  section.dataset.assessedAt=""; paragraphs[0].textContent="Unknown: decision input unavailable; open count unknown.";
  timer(); assert.match(paragraphs[0].textContent,/^Unknown:/);
}
'''
        result = subprocess.run([shutil.which("node") or "node", "-e", script, str(control.HERE / "status.js")], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_f01_published_age_initial_reload_boundaries_and_failed_health(self):
        self.value["decisions"][0]["id"] = "oldest-age"  # Legal permanent-link collision regression.
        label = 'Oldest raised 0 minutes ago. Fresh manager assessment. <b>safe</b>'
        record = self.value["decisions"][0]["revisions"][0]
        record["question"] = record["evidence"][0]["label"] = label
        self.save(self.value)
        target, _ = self.bundle()
        frozen = (target / "source" / transport.ARTIFACT).read_bytes()
        self.activate(target)
        _, page, _ = self.publish()
        self.assertIn(f'data-raised-at="{NOW}"', page)
        # Parse the actual published fragment, including its text/element children.
        class Fragment(HTMLParser):
            def __init__(self, markup):
                super().__init__()
                self.root = dict(tag="root", attrs={}, children=[])
                self.stack = [self.root]
                self.feed(markup)

            def handle_starttag(self, tag, attrs):
                node = dict(tag=tag, attrs=dict(attrs), children=[])
                self.stack[-1]["children"].append(node)
                self.stack.append(node)

            def handle_endtag(self, tag):
                assert self.stack.pop()["tag"] == tag

            def handle_data(self, text):
                self.stack[-1]["children"].append(dict(text=text))

        fragment = '<section id="decisions"' + page.split('<section id="decisions"', 1)[1].split('</section>', 1)[0] + '</section>'
        fragments = [Fragment(fragment).root]
        for status in ("resolved", "superseded"):
            markup = attention.render_decisions(revision(self.value, status), NOW, [], {})
            markup = markup.replace('<section id="decisions">',
                                    f'<section id="decisions" data-assessed-at="{NOW}" data-fresh-seconds="1200">')
            fragments.append(Fragment(markup).root)
        script = r'''
const assert = require("node:assert/strict"), vm = require("node:vm"), fs = require("node:fs");
const trees = JSON.parse(fs.readFileSync(0, "utf8"));
const source = fs.readFileSync(process.argv[1], "utf8"), base = Date.parse("2026-09-12T12:00:00Z");
const walk = n => [n, ...(n.children || []).flatMap(walk)];
function node(raw, parentElement = null) {
  const n = {...raw, parentElement, tagName:raw.tag?.toUpperCase(), dataset:{}};
  for (const [key, value] of Object.entries(raw.attrs || {})) {
    if (key.startsWith("data-")) n.dataset[key.slice(5).replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = value;
  }
  n.children = (raw.children || []).map(child => node(child, n));
  Object.defineProperties(n, {
    textContent:{get:() => n.text ?? n.children.map(c => c.textContent).join(""),
      set:value => {n.text = value; n.children = []; }},
    firstChild:{get:() => n.children[0]}, lastChild:{get:() => n.children.at(-1)},
    firstElementChild:{get:() => n.children.find(c => c.tagName)}
  });
  n.querySelectorAll = tag => walk(n).filter(c => c.tag === tag);
  n.closest = selector => selector === "#decisions" ? n.attrs?.id === "decisions" ? n : n.parentElement?.closest(selector) : null;
  return n;
}
(async () => {
for (const tree of trees) {
for (const mode of ["healthy", "failed", "offline", "http", "pending"]) {
  for (const start of [59999, 300000, 1200001]) { // Initial old snapshot/reload; stale on load.
    let now = base + start, timer, calls = 0;
    const root = node(tree), section = walk(root).find(n => n.attrs?.id === "decisions");
    const freshness = {}, lookup = id => walk(root).find(n => n.attrs?.id === id);
    const oldest = lookup("oldest-open-decision-age"), prefix = oldest?.parentElement.firstChild;
    const detail = lookup("decision-oldest-age"), history = detail.children;
    assert.equal(walk(root).filter(n => n.attrs?.id === "decision-oldest-age").length, 1);
    assert.equal(detail.tagName, "DETAILS");
    const user = walk(root).filter(n => n.tag === "a" || n.tag === "strong");
    const original = user.map(n => n.tag === "a" ? n.textContent : n.parentElement.textContent);
    const document = {body:{dataset:{generation:"same", collected:"2026-09-12T12:00:00Z"}},
      getElementById:id => id === "freshness" ? freshness : lookup(id), querySelectorAll:() => []};
    const context = vm.createContext({document, Date:{now:() => now, parse:Date.parse},
      location:{hash:"#decision-oldest-age"}, window:{addEventListener:() => {}}, setInterval:(f, ms) => {assert.equal(ms,30000); timer=f;},
      fetch:async () => {calls++; if (mode === "offline") throw Error("offline");
        if (mode === "pending") return new Promise(() => {});
        return {ok:mode !== "http", json:async () => ({ok:mode !== "failed", generation:"same", published_at:"2026-09-12T12:00:00Z"})};}});
    vm.runInContext(source, context);
    assert.equal(detail.open, true); // The permanent link opens the record, never the age span.
    timer();
    assert.equal(detail.children, history); // Age refresh must never erase decision/history markup.
    if (!oldest) {
      assert.match(detail.textContent, /Retained revision 1/);
      assert.match(detail.textContent, /Five testers/);
      assert(!detail.textContent.includes("Oldest raised age unknown."));
      continue;
    }
    assert.equal(oldest.textContent, `Oldest raised ${Math.floor(start / 60000)} minutes ago.`);
    for (const elapsed of [59999, 60000, 60001, 119999, 120000, 1200000, 1200001, 1800000]) {
      now = base + elapsed; timer();
      assert.equal(oldest.textContent, `Oldest raised ${Math.floor(elapsed / 60000)} minutes ago.`);
      assert.equal(lookup("oldest-open-decision-age"), oldest); // Stale transition cannot detach target.
    }
    assert.match(prefix.textContent, /^Last-known open: 1\./);
    const paragraphs = section.querySelectorAll("p");
    assert(paragraphs.some(p => /Stale context/.test(p.textContent)));
    assert(paragraphs.some(p => /Stale evidence/.test(p.textContent)));
    user.forEach((n, i) => { // Evidence suffix may age; user label and other text must not.
      assert.equal(n.tag === "a" ? n.textContent : n.parentElement.textContent, original[i]);
    });
    assert(paragraphs.some(p => /raised 2026-09-12T12:00:00Z; context updated 2026-09-12T12:00:00Z/.test(p.textContent)));
    assert.equal(oldest.dataset.raisedAt, "2026-09-12T12:00:00Z");
    assert.equal(section.dataset.assessedAt, "2026-09-12T12:00:00Z");
    for (const stamp of ["invalid", "", "2099-01-01T00:00:00Z"]) {
      oldest.dataset.raisedAt = stamp; timer(); assert.equal(oldest.textContent, "Oldest raised age unknown.");
    }
    assert(calls > 1);
    await new Promise(resolve => setImmediate(resolve));
    if (["offline", "http"].includes(mode)) assert.match(freshness.textContent, /Connection unavailable/);
    if (mode === "failed") assert.match(freshness.textContent, /Publisher failed/);
  }
}
}
})().catch(error => {console.error(error); process.exitCode=1;});
'''
        result = subprocess.run([shutil.which("node") or "node", "-e", script,
                                 str(self.install / "site/current/status.js")],
                                input=json.dumps(fragments), capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((target / "source" / transport.ARTIFACT).read_bytes(), frozen)


from test_slack_transport import LiveFixture


class SlackProjectionTests(LiveFixture):
    def test_projection_observes_live_owner_and_lease_without_mutating_journals(self):
        import slack_transport as slack
        self.sending()
        journals = {name: (self.root / name).read_bytes() for name in
                    ("alerts.json", "replies.json", "transport.json", ".ingress.fence")}
        with patch("slack_sdk.WebClient", side_effect=AssertionError("no network")):
            self.assertEqual(transport.project_slack(self.feed_root, self.root, NOW, True)["status"]["state"], "last-verified")
            expired = slack.time.monotonic_ns() + slack.LEASE_NS
            with patch.object(slack.time, "monotonic_ns", return_value=expired):
                self.assertEqual(transport.project_slack(self.feed_root, self.root, NOW, True)["status"]["state"], "held")
            self.owner.release()  # Leave durable connected metadata intact: ownership is gone.
            self.assertEqual(transport.project_slack(self.feed_root, self.root, NOW, True)["status"]["state"], "held")
        self.assertEqual(journals, {name: (self.root / name).read_bytes() for name in journals})

    def test_actual_ingress_two_questions_revision_mapping_and_no_network_projection(self):
        import decision_alerts as alerts
        import slack_transport as slack
        from test_decision_alerts import PIN, OWNER, REMOTE
        old = copy.deepcopy(self.feed)
        second = copy.deepcopy(self.feed["decisions"][0])
        second["id"] = "choice-2"
        self.feed["decisions"].append(second)
        self.feed["revision"] += 1
        d.save_fixture(self.feed_root, self.feed, d.digest(old), NOW)
        alerts.enqueue(self.store, self.feed, "choice-2", REMOTE, PIN, OWNER, NOW)
        self.sending()
        self.callback()
        with patch("slack_sdk.WebClient", side_effect=AssertionError("projection must not connect")):
            pending = transport.project_slack(self.feed_root, self.root, NOW, True)
        rows = {row["id"]: row for row in pending["decisions"]}
        self.assertEqual((rows["choice-1"]["delivery"], rows["choice-1"]["pending"]), ("sent", 1))
        self.assertEqual((rows["choice-2"]["delivery"], rows["choice-2"]["pending"]), ("pending", 0))
        self.assertEqual(slack.drain(self.store), 1)
        journals = {name: (self.root / name).read_bytes() for name in ("alerts.json", "replies.json", "transport.json", ".ingress.fence")}
        calls = len(self.calls)
        value = transport.project_slack(self.feed_root, self.root, NOW, True)
        rows = {row["id"]: row for row in value["decisions"]}
        self.assertEqual(rows["choice-1"]["replies"]["received"], 1)
        self.assertFalse(any(rows["choice-2"]["replies"].values()))
        self.assertEqual(rows["choice-1"]["pending"], 0)
        self.assertNotIn("Five testers", d.canonical(value).decode())
        self.assertNotIn("fixture-bot", d.canonical(value).decode())
        self.assertEqual(calls, len(self.calls))
        self.assertEqual(journals, {name: (self.root / name).read_bytes() for name in journals})
        changed = revision(self.feed, "resolved")
        d.save_fixture(self.feed_root, changed, d.digest(self.feed), NOW)
        raw, _ = transport.capture(self.feed_root, NOW)
        self.assertEqual(json.loads(raw)["slack_status"], "invalid")
        latest = transport.project_slack(self.feed_root, self.root, NOW, True)
        rows = {(row["id"], row["revision"]): row for row in latest["decisions"]}
        self.assertEqual(rows[("choice-1", 1)]["replies"]["received"], 1)
        self.assertFalse(any(rows[("choice-1", 2)]["replies"].values()))
        page = transport.render(dict(feed=changed, slack=latest, slack_status="valid"), NOW, [], {})
        self.assertIn("Slack — question revision 1", page)
        self.assertIn("received: 1", page)
        slack.hold(self.store, "outage-gap")
        self.assertEqual(transport.project_slack(self.feed_root, self.root, NOW, True)["status"]["state"], "held")

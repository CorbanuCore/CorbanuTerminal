import copy
import hashlib
import json
import multiprocessing
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import decisions as d

NOW = "2026-09-12T12:00:00Z"
LATER = "2026-09-12T12:21:00Z"
SPRINT_PATH = "docs/sprints/current/delivery.md"


def fixture():
    return {"schema": 1, "feed_id": "manager-fixture", "revision": 1, "assessed_at": NOW,
            "decisions": [{"id": "choice-1", "revisions": [{
                "revision": 1, "initiative": "PF-80", "sprints": [
                    {"sprint_id": "PF-80-S01", "path": SPRINT_PATH, "historical": False}],
                "raised_at": NOW, "updated_at": NOW, "summary": "PF-80-S01 pilot size",
                "owner": "Travis", "background": "Synthetic pilot planning",
                "impact": "Ten testers increase triage", "stopped": "Pilot invitations",
                "continuing": "Offline implementation", "options": ["Five testers", "Ten testers"],
                "recommendation": "Five testers bounds triage", "question": "Five or ten testers?",
                "evidence": [{"label": "Sanitized test plan", "path": SPRINT_PATH, "assessed_at": NOW}],
                "status": "open", "resolution": None}]}]}


def revision(feed, status="open"):
    result = copy.deepcopy(feed)
    result["revision"] += 1
    history = result["decisions"][0]["revisions"]
    record = copy.deepcopy(history[-1])
    record.update(revision=len(history) + 1, status=status, resolution=None)
    if status in ("resolved", "superseded"):
        record["resolution"] = {"answered_revision": len(history), "actor": "Travis",
                                "answer": "Five testers", "scope": "Synthetic pilot only", "recorded_at": NOW}
    history.append(record)
    return result


def compete(root, value, expected, barrier, results):
    barrier.wait(timeout=10)
    try:
        d.save_fixture(root, value, expected, NOW)
        results.put("saved")
    except d.Conflict:
        results.put("conflict")


class DecisionTests(unittest.TestCase):
    def test_complete_roundtrip_and_canonical_digest(self):
        value = fixture()
        self.assertEqual(d.validate(json.dumps(value).encode(), NOW), value)
        self.assertIsNot(d.validate(value, NOW), value)
        self.assertEqual(d.digest({"b": 2, "a": 1}), hashlib.sha256(b'{"a":1,"b":2}').hexdigest())
        self.assertEqual(d.project(value, NOW), {"state": "fresh", "feed": value, "open": ["choice-1"]})

    def test_all_lifecycle_states_and_exact_answer_revision(self):
        for status in ("open", "acknowledged", "resolved", "superseded"):
            with self.subTest(status=status):
                value = revision(fixture(), status)
                self.assertEqual(d.validate(value, NOW), value)
                self.assertEqual(d.project(value, NOW)["open"], ["choice-1"] if status in ("open", "acknowledged") else [])
        value = revision(fixture())
        value["decisions"][0]["revisions"][-1]["question"] = "When should the pilot begin?"
        self.assertEqual(d.validate(value, NOW), value)
        resolved = revision(value, "resolved")
        resolved["decisions"][0]["revisions"][-1]["resolution"]["answered_revision"] = 1
        with self.assertRaises(d.Invalid):
            d.validate(resolved, NOW)
        reopened = revision(revision(value, "resolved"))
        reopened["decisions"][0]["revisions"][-1]["question"] = "Should a later pilot expand?"
        self.assertEqual(d.validate(reopened, NOW), reopened)

    def test_old_answer_cannot_resolve_changed_or_reopened_context(self):
        for status in ("resolved", "superseded"):
            for field, changed in (("options", ["Twenty testers", "Thirty testers"]),
                                   ("stopped", "Production rollout"),
                                   ("sprints", [dict(sprint_id="PF-60-S02", path=SPRINT_PATH, historical=False)])):
                value = revision(fixture())
                value["decisions"][0]["revisions"][-1][field] = changed
                value = revision(value, status)
                value["decisions"][0]["revisions"][-1]["resolution"]["answered_revision"] = 1
                with self.subTest(status=status, field=field), self.assertRaises(d.Invalid):
                    d.validate(value, NOW)
            value = revision(revision(revision(fixture(), "resolved")), status)
            value["decisions"][0]["revisions"][-1]["resolution"]["answered_revision"] = 1
            with self.assertRaises(d.Invalid):
                d.validate(value, NOW)
        same = revision(revision(fixture(), "acknowledged"), "resolved")
        same["decisions"][0]["revisions"][-1]["resolution"]["answered_revision"] = 1
        self.assertEqual(d.validate(same, NOW), same)

    def test_canonical_expansion_cannot_replace_last_good_fixture(self):
        value = revision(fixture())
        value["decisions"][0]["revisions"][-1]["options"] = ["界" * 600 + str(i) for i in range(700)]
        raw = json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode("utf-16")
        self.assertLess(len(raw), d.MAX_BYTES)
        self.assertGreater(len(d.canonical(value)), d.MAX_BYTES)
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            first = fixture()
            token = d.save_fixture(root, first, None, NOW)
            original = (Path(root) / "decisions.fixture.json").read_bytes()
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, raw, token, NOW)
            self.assertEqual((Path(root) / "decisions.fixture.json").read_bytes(), original)
            self.assertEqual(d.load_fixture(root, NOW), first)

    def test_unknown_empty_stale_boundary_and_new_assessment(self):
        unknown = {"state": "unknown", "feed": None, "open": None}
        for raw in (None, b'{}', b'{"schema":1,"schema":1}', b'[]', b'NaN'):
            self.assertEqual(d.project(raw, NOW), unknown)
        value = fixture()
        self.assertEqual(d.project(value, "2026-09-12T12:20:00Z")["state"], "fresh")
        self.assertEqual(d.project(value, "2026-09-12T12:20:01Z")["state"], "stale")
        value["decisions"] = []
        self.assertEqual(d.project(value, LATER), {"state": "stale", "feed": value, "open": []})
        value.update(revision=2, assessed_at=LATER)
        self.assertEqual(d.project(value, LATER), {"state": "fresh", "feed": value, "open": []})

    def test_schema_time_bounds_references_and_safe_errors(self):
        bad = []
        for field, value in (("extra", "synthetic-secret"), ("schema", True), ("revision", False), ("revision", 0),
                             ("assessed_at", LATER), ("assessed_at", "2026-02-30T12:00:00Z"),
                             ("assessed_at", "2026-09-12T12:00:00"), ("feed_id", "../escape")):
            feed = fixture()
            feed[field] = value
            bad.append(feed)
        for field, value in (("revision", 2), ("owner", "xoxb-synthetic-canary"), ("summary", "x" * 301),
                             ("background", "RAW_PRIVATE_LOG canary"), ("impact", "secret=CANARY"),
                             ("question", "bad\x7f"), ("question", "bad\u202e"), ("updated_at", LATER),
                             ("status", "approved"), ("options", ["Only one"]), ("resolution", {})):
            feed = fixture()
            feed["decisions"][0]["revisions"][0][field] = value
            bad.append(feed)
        for path in ("../escape", "/tmp/private", "docs/sprints/../private.md", "docs/sprints//x.md", "https://host/x", "qa/raw_private_log.md"):
            feed = fixture()
            feed["decisions"][0]["revisions"][0]["sprints"][0]["path"] = path
            bad.append(feed)
        duplicate = fixture()
        duplicate["decisions"] *= 2
        bad.extend([duplicate, b'"' + b'x' * d.MAX_BYTES + b'"', b'[[[' * 2000])
        for raw in bad:
            with self.subTest(raw_type=type(raw).__name__), self.assertRaisesRegex(d.Invalid, "^Decision input unavailable: invalid fixture.$"):
                d.validate(raw, NOW)

    def test_append_only_persistence_retry_reload_and_rollback(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            self.assertIsNone(d.load_fixture(root, NOW))
            first = fixture()
            token = d.save_fixture(root, first, None, NOW)
            for status in ("open", "acknowledged", "resolved"):
                next_value = revision(first, status)
                token = d.save_fixture(root, next_value, token, NOW)
                self.assertEqual(d.load_fixture(root, NOW), next_value)
                self.assertEqual(d.save_fixture(root, next_value, None, LATER), token)
                first = next_value
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, fixture(), token, NOW)
            for mutation in ("delete", "rewrite", "feed", "assessment", "gap"):
                changed = copy.deepcopy(first)
                changed["revision"] += 1
                if mutation == "delete":
                    changed["decisions"] = []
                elif mutation == "rewrite":
                    changed["decisions"][0]["revisions"][0]["summary"] = "Rewritten history"
                elif mutation == "feed":
                    changed["feed_id"] = "another-source"
                elif mutation == "assessment":
                    changed["assessed_at"] = "2026-09-12T11:59:00Z"
                else:
                    changed["revision"] += 1
                with self.assertRaises(d.Invalid):
                    d.save_fixture(root, changed, token, NOW)
                self.assertEqual(d.load_fixture(root, NOW), first)

    def test_concurrent_process_cas_has_one_winner(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            first = fixture()
            token = d.save_fixture(root, first, None, NOW)
            choices = [revision(first), revision(first, "acknowledged")]
            ctx = multiprocessing.get_context("spawn")
            barrier, results = ctx.Barrier(2), ctx.Queue()
            workers = [ctx.Process(target=compete, args=(root, value, token, barrier, results)) for value in choices]
            for worker in workers:
                worker.start()
            for worker in workers:
                worker.join(15)
                self.assertFalse(worker.is_alive())
                self.assertEqual(worker.exitcode, 0)
            self.assertEqual(sorted(results.get(timeout=2) for _ in workers), ["conflict", "saved"])
            self.assertIn(d.load_fixture(root, NOW), choices)
            results.close()

    def test_pre_replace_failure_retains_bytes_and_private_modes(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            value = fixture()
            token = d.save_fixture(root, value, None, NOW)
            path = Path(root) / "decisions.fixture.json"
            original = path.read_bytes()
            for operation in ("replace", "fsync"):
                with patch.object(d.os, operation, side_effect=OSError("synthetic-secret")), self.assertRaises(d.Invalid):
                    d.save_fixture(root, revision(value), token, NOW)
                self.assertEqual(path.read_bytes(), original)
                self.assertEqual(d.load_fixture(root, NOW), value)
                self.assertEqual(list(Path(root).glob(".decisions-pending-*")), [])
            for entry in Path(root).iterdir():
                self.assertEqual(entry.stat().st_mode & 0o777, 0o600)

    def test_unsafe_fixture_files_and_directory_reject(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            path = Path(root) / "decisions.fixture.json"
            path.symlink_to(Path(root) / "absent")
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, fixture(), None, NOW)
            path.unlink()
            lock = Path(root) / ".decisions.fixture.lock"
            lock.unlink()
            lock.symlink_to(Path(root) / "absent")
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, fixture(), None, NOW)
            os.chmod(root, 0o755)
            with self.assertRaises(d.Invalid):
                d.load_fixture(root, NOW)

    def test_corrupt_or_public_fixture_cannot_be_overwritten_as_missing(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as root:
            value = fixture()
            token = d.save_fixture(root, value, None, NOW)
            path = Path(root) / "decisions.fixture.json"
            os.chmod(path, 0o644)
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, revision(value), token, NOW)
            os.chmod(path, 0o600)
            path.write_bytes(b'{"secret":"SYNTHETIC_SECRET", "secret": "duplicate"}')
            original = path.read_bytes()
            with self.assertRaises(d.Invalid):
                d.load_fixture(root, NOW)
            with self.assertRaises(d.Invalid):
                d.save_fixture(root, value, None, NOW)
            self.assertEqual(path.read_bytes(), original)

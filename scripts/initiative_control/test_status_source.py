import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import control
import export
import sync_check
from test_control import run


class StatusSourceTests(unittest.TestCase):
    def setUp(self):
        self.sprint = dict(status="in_progress", sprint_id="PF-80-S01", path="docs/sprints/current/task.md")
        self.data = dict(runs=[], config={"human_tests": []})

    def test_latest_note_and_own_sprint_only(self):
        self.data["runs"] = [
            {**run(), "updated_at": "2026-01-01T00:00:00Z", "summary": "Old resolved question"},
            {**run(), "summary": "Native validity returned; manager review pending."},
            {**run(), "sprint_id": "PF-60-S02", "summary": "Wrong initiative"},
        ]
        label, details = control.sprint_status(self.sprint, self.data)
        self.assertIn('href="#status-PF-80-S01"', label)
        self.assertIn('role="tooltip"', label)
        self.assertIn("Native validity returned", label)
        self.assertNotIn("Old resolved question", label + details)
        self.assertNotIn("Wrong initiative", label + details)

    def test_blockers_link_and_all_reasons_escape(self):
        self.sprint["status"] = "blocked"
        self.data["config"]["human_tests"] = [
            dict(sprint_id="PF-80-S01", status="blocked", reason="Await exact approved payload"),
            dict(sprint_id="PF-80-S01", status="blocked", reason="<script>not executable</script>"),
            dict(sprint_id="PF-60-S02", status="blocked", reason="Wrong initiative"),
        ]
        label, details = control.sprint_status(self.sprint, self.data)
        self.assertIn('href="#status-PF-80-S01"', label)
        self.assertIn('id="status-PF-80-S01"', details)
        self.assertIn("Await exact approved payload", details)
        self.assertIn("&lt;script&gt;", details)
        self.assertNotIn("<script>", details)
        self.assertNotIn("Wrong initiative", details)

    def test_missing_and_stale_never_fabricate_progress(self):
        label, _ = control.sprint_status(self.sprint, self.data)
        self.assertIn("No progress report connected", label)
        self.data["runs"] = [{**run(), "updated_at": "2026-01-01T00:00:00Z", "summary": "A" * 500}]
        label, details = control.sprint_status(self.sprint, self.data)
        self.assertIn("STALE", label)
        self.assertNotIn("A" * 500, label)
        self.assertIn("A" * 500, details)

    def test_resolved_question_and_posting_off_do_not_override_manager(self):
        self.data["config"].update(tasknode={"enabled": False}, human_tests=[
            dict(sprint_id="PF-80-S01", status="pending", reason="Publisher approved")])
        self.data["runs"] = [{**run(), "status": "blocked", "summary": "Old question"}]
        label, details = control.sprint_status(self.sprint, self.data)
        self.assertIn('class="badge in_progress"', label)
        self.assertNotIn("Block reasons", details)
        self.assertFalse(self.data["config"]["tasknode"]["enabled"])

    def test_blocked_without_note_links_full_record_and_admits_missing(self):
        self.sprint["status"] = "blocked"
        _, details = control.sprint_status(self.sprint, self.data)
        self.assertIn("No condensed blocker recorded", details)
        self.assertIn(control.route(self.sprint["path"]), details)

    def test_source_verification_detects_wrong_branch_revision_content_and_checkout(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            (repo / "plan.md").write_text("current")
            manifest = dict(commit="a" * 40, branch="manager", checkout=str(repo.resolve()),
                            files={"plan.md": hashlib.sha256(b"current").hexdigest()})
            with patch.object(export, "identity", return_value=("a" * 40, "manager")):
                export.verify_source(repo, manifest, "manager")
                with self.assertRaisesRegex(ValueError, "declared manager branch"):
                    export.verify_source(repo, manifest, "old-recovery")
                with self.assertRaisesRegex(ValueError, "revision changed"):
                    export.verify_source(repo, {**manifest, "commit": "b" * 40})
                with self.assertRaisesRegex(ValueError, "checkout differs"):
                    export.verify_source(repo, {**manifest, "checkout": "/obsolete"})
                (repo / "plan.md").write_text("newer")
                with self.assertRaisesRegex(ValueError, "contents changed"):
                    export.verify_source(repo, manifest)

    def test_export_rejects_mid_collection_changes_without_advancing_source_time(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, state = root / "repo", root / "state"
            repo.mkdir()
            (repo / "plan.md").write_text("plan")
            control.atomic_json(state / "control.json", {"human_tests": []})
            control.atomic_json(state / "source.json", {"collected_at": "last-good"})
            with patch.object(export, "identity", return_value=("a" * 40, "manager")), patch.object(export, "source_paths", return_value={repo / "plan.md"}), patch.object(export, "verify_source", side_effect=ValueError("changed")):
                with self.assertRaises(ValueError):
                    export.export(repo, state, root / "bundle", "manager")
            self.assertEqual(json.loads((state / "source.json").read_text())["collected_at"], "last-good")

    def test_postflight_rejects_stale_publication_and_failure_keeps_last_good(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            generation = root / "site/releases/build-test"
            generation.mkdir(parents=True)
            (generation / "index.html").write_text("last-good")
            (root / "site/current").symlink_to(generation)
            source = dict(commit="a" * 40, branch="manager", tree_digest="b" * 64, collected_at=control.now())
            control.atomic_json(root / "state/source.json", source)
            control.atomic_json(generation / "manifest.json", {"source": source})
            health = dict(ok=True, generation="build-test", collected_at=source["collected_at"])
            control.atomic_json(root / "site/health.json", health)
            sync_check.check(root)
            with self.assertRaisesRegex(ValueError, "expected export"):
                sync_check.check(root, expected_commit="c" * 40)
            with self.assertRaisesRegex(ValueError, "expected export"):
                sync_check.check(root, expected_digest="d" * 64)
            control.atomic_json(root / "state/source.json", {**source, "commit": "c" * 40})
            with self.assertRaises(ValueError):
                sync_check.check(root)
            sync_check.failed(root)
            self.assertFalse(json.loads((root / "site/health.json").read_text())["ok"])
            self.assertEqual((root / "site/current/index.html").read_text(), "last-good")
            control.atomic_json(root / "state/source.json", source)
            control.atomic_json(root / "site/health.json", health)
            sync_check.check(root)

    def test_render_timer_cannot_clear_a_failed_source_sync(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            old = root / "old"
            old.mkdir()
            (old / "index.html").write_text("last-good")
            (root / "current").symlink_to(old)
            control.atomic_json(root / "sync-failure.json", {"attempted_at": control.now()})
            data = {"source": {"collected_at": "2026-01-01T00:00:00Z"}}
            with patch.object(control, "collect", return_value=data), self.assertRaisesRegex(ValueError, "source synchronization failed"):
                control.publish(root, root, root)
            self.assertEqual((root / "current/index.html").read_text(), "last-good")


if __name__ == "__main__":
    unittest.main()

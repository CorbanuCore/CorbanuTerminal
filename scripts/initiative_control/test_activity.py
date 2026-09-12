import datetime as dt
import unittest

import activity
import control
from test_control import run


class ActivityTests(unittest.TestCase):
    def setUp(self):
        self.sprint = dict(status="in_progress", sprint_id="PF-80-S01", path="docs/sprints/current/task.md")

    def render(self, reports):
        return control.sprint_status(self.sprint, dict(runs=reports, config={"human_tests": []}))

    def test_review_uses_same_primary_wording_as_run_and_secondary_lifecycle(self):
        report = {**run(), "status": "awaiting_review"}
        label, _ = self.render([report])
        self.assertIn(control.activity_presentation([report])[0], label)
        self.assertIn(">Awaiting manager review</a>", label)
        self.assertIn('class="badge awaiting_review"', label)
        self.assertIn("Sprint lifecycle: in progress", label)
        self.assertNotIn('class="badge in_progress"', label)

    def test_working_to_returned_does_not_complete_sprint(self):
        working = {**run(), "updated_at": "2026-01-01T00:00:00Z"}
        returned = {**run(), "status": "awaiting_review"}
        self.assertEqual(activity.presentation([working])[0], "Working")
        label, details = self.render([working, returned])
        self.assertIn(">Awaiting manager review</a>", label)
        self.assertNotIn("completed", label)
        self.assertEqual(self.sprint["status"], "in_progress")

    def test_missing_and_stale_do_not_claim_live_execution(self):
        self.assertIn("Activity unknown", self.render([])[0])
        for status in activity.LABELS:
            label, _ = self.render([{**run(), "status": status, "updated_at": "2026-01-01T00:00:00Z"}])
            self.assertIn("Stale report — last:", label)
            self.assertIn('class="badge stale"', label)

    def test_mixed_workers_and_terminal_history(self):
        working = run()
        review = {**run(), "run_id": "review", "status": "awaiting_review"}
        returned = {**run(), "run_id": "old", "status": "finished"}
        self.assertEqual(activity.presentation([working, review])[0], "Mixed activity — see run details")
        self.assertEqual(activity.presentation([working, returned])[0], "Working")
        label, detail = self.render([working, review])
        self.assertIn("Mixed activity", label)
        self.assertIn("review · Awaiting manager review", detail)
        self.assertIn("run-1 · Working", detail)

    def test_equal_time_conflicts_preserved_without_arbitrary_precedence(self):
        a = run()
        b = {**a, "status": "awaiting_review", "summary": "Conflicting observation"}
        for reports in ([a, b], [b, a]):
            self.assertEqual(len(activity.latest_reports(reports)), 2)
            self.assertEqual(activity.presentation(reports)[0], "Conflicting reports — manager check")
            label, detail = self.render(reports)
            self.assertIn("no reliable latest note", label)
            self.assertIn("Conflicting observation", detail)
        self.assertEqual(len(activity.latest_reports([a, a])), 1)

    def test_manager_blocker_link_preserved_despite_working_report(self):
        self.sprint["status"] = "blocked"
        label, detail = self.render([run()])
        self.assertIn('href="#status-PF-80-S01"', label)
        self.assertIn("Block reasons", detail)
        self.assertIn("Sprint lifecycle: blocked", label)

    def test_table_conflict_warning_is_outside_age_updated_badge(self):
        a = {**run(), "updated_at": "2026-01-01T00:00:00Z"}
        b = {**a, "status": "awaiting_review"}
        data = dict(plans={"plans": [], "active_limit": 3}, sprints={"sprints": []},
                    config={}, source={"label": "fixture", "branch": "fixture", "commit": "a"*40},
                    problems=[], runs=[a], display_runs=[a, b], events=[])
        page = control.overview(data)
        self.assertEqual(page.count("</span><small>Conflicting same-time reports"), 2)
        self.assertEqual(data["runs"], [a])  # Rendering never changes delivery selection.

    def test_mixed_fresh_and_stale_unresolved_runs_remain_qualified(self):
        old = {**run(), "run_id": "old", "updated_at": "2026-01-01T00:00:00Z"}
        label, _ = self.render([old, run()])
        self.assertIn("Stale report", label)

    def test_staleness_boundary_and_client_timer_share_45_minutes(self):
        current = control.timestamp(control.now())
        recent = {**run(), "updated_at": (current - dt.timedelta(minutes=44)).isoformat()}
        self.assertNotIn("Stale report", self.render([recent])[0])
        self.assertIn("data-activity-seen=", self.render([recent])[0])
        script = (control.HERE / "status.js").read_text()
        self.assertIn("45 * 60000", script)
        self.assertIn("updateActivityAge();", script)


if __name__ == "__main__":
    unittest.main()

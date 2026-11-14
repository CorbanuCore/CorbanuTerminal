"""Round-68 failing-first reproduction; current regressions live in test_decision_feed.

Run with checkout scripts/initiative_control on PYTHONPATH in the isolated venv.
These are synthetic offline cases; they never read a live feed or Slack store.
"""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import control
import fable_launcher as f
import decision_feed as feed
import decisions as d
from test_decisions import NOW, fixture, revision


def growing_feed():
    value = fixture()
    three = revision(revision(value))["decisions"][0]["revisions"]
    value["decisions"] = [
        dict(id=f"question-{index:02}", revisions=copy.deepcopy(three[:3 if index < 13 else 2]))
        for index in range(20)
    ]
    return d.validate(value, NOW)


class ProjectionRegressionTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve())
        self.addCleanup(self.tmp.cleanup)
        self.state = Path(self.tmp.name)
        f.write_file(self.state / ".decisions.fixture.lock", b"")

    def save(self, value):
        # Fixtures are fully validated; no live feed lineage is manufactured.
        control.atomic_json(self.state / "decisions.fixture.json", d.validate(value, NOW))

    def test_twenty_decisions_fifty_three_revisions_projects_within_bound(self):
        value = growing_feed()
        self.assertEqual(20, len(value["decisions"]))
        self.assertEqual(53, sum(len(item["revisions"]) for item in value["decisions"]))
        self.save(value)
        result = feed.project_slack(self.state, None, NOW)
        self.assertLessEqual(len(d.canonical(result)), feed.SLACK_LIMIT)
        selected = {(row["id"], row["revision"]) for row in result["decisions"]}
        self.assertTrue(all((item["id"], item["revisions"][-1]["revision"]) in selected
                            for item in value["decisions"]))

    def test_bound_never_drops_open_or_acknowledged_question_after_closed_history(self):
        value = fixture()
        resolved = revision(value, "resolved")["decisions"][0]["revisions"]
        value["decisions"] = [
            dict(id=f"closed-{index:03}", revisions=copy.deepcopy(resolved))
            for index in range(80)
        ]
        for status in ("open", "acknowledged"):
            history = fixture()["decisions"][0]["revisions"] if status == "open" else revision(
                fixture(), "acknowledged")["decisions"][0]["revisions"]
            value["decisions"].append(dict(id="last-" + status, revisions=history))
        self.save(value)
        result = feed.project_slack(self.state, None, NOW)
        self.assertLessEqual(len(d.canonical(result)), feed.SLACK_LIMIT)
        selected = {(row["id"], row["revision"]) for row in result["decisions"]}
        self.assertIn(("last-open", 1), selected, "latest open question missing")
        self.assertIn(("last-acknowledged", 2), selected, "latest acknowledged question missing")

    def test_oversize_fails_before_cache_write_without_silent_row_truncation(self):
        self.save(fixture())
        feed.project_slack(self.state, None, NOW)
        before = (self.state / feed.SLACK_FILE).read_bytes()
        value = growing_feed()
        self.save(value)
        observed = {}
        validate = feed.validate_slack

        def capture(result, current, at):
            observed.update(rows=len(result["decisions"]), bytes=len(d.canonical(result)))
            return validate(result, current, at)

        # Force overflow even after the retention fix; this remains a
        # diagnostic of fail-closed cache writes, not the 20/53 regression.
        with patch.object(feed, "validate_slack", side_effect=capture), patch.object(feed, "SLACK_LIMIT", 100):
            with self.assertRaises(d.Invalid):
                feed.project_slack(self.state, None, NOW)
        self.assertEqual(20, observed["rows"])
        self.assertGreater(observed["bytes"], 100)
        self.assertEqual(before, (self.state / feed.SLACK_FILE).read_bytes())
        print("Synthetic 20-decision/53-revision projection:", observed)


if __name__ == "__main__":
    unittest.main(verbosity=2)

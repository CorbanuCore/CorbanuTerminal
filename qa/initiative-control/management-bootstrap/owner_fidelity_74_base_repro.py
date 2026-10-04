"""Replay final regression cases against frozen base modules, without checkout edits."""
import importlib
from pathlib import Path
import subprocess
import unittest

BASE = "d778cba2aedc3dac21e71266e23c346895afac2c"
for name in ("decision_feed", "attention", "owner_daemon"):
    module = importlib.import_module(name)
    path = Path(module.__file__).resolve().relative_to(Path.cwd())
    source = subprocess.run(["git", "show", f"{BASE}:{path}"],
                            capture_output=True, check=True, text=True).stdout
    exec(compile(source, str(path), "exec"), module.__dict__)

cases = [
    "test_decision_feed.SlackProjectionTests.test_resolved_decision_slack_line_survives_omission_with_slack_on",
    "test_decision_feed.SlackProjectionTests.test_resolved_decision_slack_line_survives_omission_with_slack_off",
    "test_decision_feed.ProjectionRegressionTests.test_historical_resolutions_also_retain_their_answered_revisions",
    "test_attention.DecisionRenderingTests.test_omitted_settled_history_is_distinct_from_unavailable_status",
    "test_owner_daemon.ActivationVisibilityTests.test_partial_status_cli_redacts_untrusted_error_details",
]
suite = unittest.defaultTestLoader.loadTestsFromNames(cases)
result = unittest.TextTestRunner(verbosity=2).run(suite)
raise SystemExit(not result.wasSuccessful())

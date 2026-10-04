"""Disposable offline partial-status and projection measurements."""
import json

import decision_feed as feed
import fable_launcher as f
import owner_daemon as owner
from test_owner_daemon import OwnerDaemonTests
from test_decision_feed import ProjectionRegressionTests, growing_feed
from test_decisions import NOW

case = OwnerDaemonTests()
case.setUp()
try:
    f.write_file(case.root / "coordinator.sqlite3-journal", b"fixture hot-journal marker")
    result = owner.activation_status(case.config_path)
    assert result["state"] == "off"
    assert result["complete"] is False
    assert result["coordinator"] is None
    assert result["unavailable"]["coordinator"]["reason"] == "coordinator_recovery_required"
    print(json.dumps(dict(partial_status=result), indent=2))
finally:
    case.tearDown()

case = ProjectionRegressionTests()
case.setUp()
try:
    case.save(growing_feed())
    result = feed.project_slack(case.state, None, NOW)
    print(json.dumps(dict(
        projection_rows=len(result["decisions"]), omitted_revisions=result["omitted_revisions"],
        canonical_bytes=len(feed.d.canonical(result)),
        disk_bytes=(case.state / feed.SLACK_FILE).stat().st_size,
        minimum_record_bytes=feed.MIN_RECORD_BYTES, maximum_rows=feed.MAX_SLACK_ROWS,
        slack_limit=feed.SLACK_LIMIT, transport_limit=feed.LIMIT), indent=2))
finally:
    case.doCleanups()

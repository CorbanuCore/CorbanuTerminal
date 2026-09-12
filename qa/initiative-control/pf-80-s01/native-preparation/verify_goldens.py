"""Offline cross-language checks; use the existing pinned Python environment."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts/initiative_control"))
import tasknode

source = (ROOT / "codex-rs/tasknode-session/src/delivery_goal_tests.rs").read_text()
fixtures = re.findall(r'const (ASCII|UNICODE): &str = r#"(.*?)"#;', source)
assert len(fixtures) == 2
contract = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/tasknode-beta.Rf2AJJ/tasknode-source/server/campaign-tracker-contract.js")
assert hashlib.sha256(contract.read_bytes()).hexdigest() == "a2f11df4adca1abe09b70bc4a0d7ae36118501831b13e884bc9e5de8c7198a7c"
for name, raw in fixtures:
    event = json.loads(raw)
    expected = re.search(rf'const {name}_DIGEST: &str =\s*"([a-f0-9]+)";', source)[1]
    assert json.dumps(event, sort_keys=True) == raw
    assert hashlib.sha256(json.dumps({"event": event}, sort_keys=True).encode()).hexdigest() == expected
    with patch.object(tasknode, "now", return_value="2026-09-11T12:00:00Z"):
        assert tasknode.checked_event(event, event["id"]) == event
        # Exercise actual Python prepare without filesystem, credentials or network.
        records = [{"event": event, "status": "pending", "next_attempt_at": event["occurredAt"]},
                   {"tasknode": {"enabled": False, "workspace_id": event["workspaceId"],
                                 "task_mappings": {event["turnId"]: event["taskIds"]}}}]
        with patch.object(tasknode, "read_json", side_effect=records), patch.object(tasknode, "enrolled", return_value=False):
            advisory = tasknode.prepare(Path("/synthetic/never-read"), event["id"])
        assert advisory["payload"] == {"event": event}
        assert advisory["send_authorized"] is False
        assert advisory["blockers"] == ["live_authority_entitlement_owner_and_target_lifecycle_unverified",
                                        "posting_disabled", "local_workspace_enrollment_unverified"]
    script = """import {validateEvent} from %s;
let raw = ''; for await (const chunk of process.stdin) raw += chunk;
Date.now = () => Date.parse('2026-09-11T12:00:00Z');
const event = JSON.parse(raw); const checked = validateEvent(event);
for (const key of Object.keys(event)) {
  if (JSON.stringify(checked[key]) !== JSON.stringify(event[key])) throw Error(key);
}
console.log('pinned validateEvent: PASS');""" % json.dumps(contract.as_uri())
    subprocess.run(["node", "--input-type=module", "-e", script], input=raw, text=True, check=True)
    print(f"{name}: Python checked_event + prepare, full-payload SHA-256, pinned server: PASS")
print("2 cross-language golden cases passed; no live activity")

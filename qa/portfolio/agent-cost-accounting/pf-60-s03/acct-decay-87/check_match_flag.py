"""Exercise the replay receipt assertion without executing any package or tests."""
import copy
import importlib.util
import json
from pathlib import Path

here = Path(__file__).resolve().parent
script = here.parent / "acct-selfcheck-84/check_current_tip.py"
spec = importlib.util.spec_from_file_location("current_tip_replay", script)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
receipt = json.loads((here / "current-tip/receipt.json").read_text())
module.assert_receipt(receipt)
results = [dict(case="actual-current-tip", passed=True)]
for field, value in (("stdout_matches_prior_replay", False),
                     ("source_tip_after", "changed-integration-tip")):
    changed = copy.deepcopy(receipt)
    changed[field] = value
    try:
        module.assert_receipt(changed)
    except AssertionError as exc:
        results.append(dict(case=field, passed=True, rejection=str(exc)))
    else:
        raise AssertionError("receipt accepted mutation: " + field)
(here / "match-flag-checks.json").write_text(json.dumps(results, indent=2) + "\n")
print(json.dumps(results, indent=2))

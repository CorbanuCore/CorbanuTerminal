"""Reject isolated violations of all five replay-receipt assertions."""
import copy
import importlib.util
import json
import sys
from pathlib import Path

here = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "current_tip_replay", here.parent / "acct-selfcheck-84/check_current_tip.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
receipt = json.loads((here.parent / "acct-reference-88/advanced/receipt.json").read_text())
module.assert_receipt(receipt)
results = [dict(case="actual-advanced-tip", passed=True)]
mutations = [
    ("output-match", ("stdout_matches_prior_replay",), False, "differs from pinned replay"),
    ("stable-tip", ("source_tip_after",), "changed-tip", "tip advanced"),
    ("clean-checkout-before", ("status_before",), " M synthetic", "checkout changed"),
    ("clean-checkout-after", ("status_after",), " M synthetic", "checkout changed"),
    ("clean-checkout-both", (), None, "checkout changed"),
]
for index, row in enumerate(receipt["commands"]):
    mutations.extend([
        ("exit-" + row["name"], ("commands", index, "exit"), 999, "unexpected command exit"),
        ("stderr-" + row["name"], ("commands", index, "stderr_empty"), False, "command wrote stderr"),
    ])
for name, path, value, reason in mutations:
    changed = copy.deepcopy(receipt)
    if path:
        destination = changed
        for key in path[:-1]:
            destination = destination[key]
        destination[path[-1]] = value
    else:
        changed["status_before"] = changed["status_after"] = " M synthetic"
    try:
        module.assert_receipt(changed)
    except AssertionError as exc:
        if reason not in str(exc):
            raise RuntimeError((name, str(exc))) from exc
        results.append(dict(case=name, mutation_path=list(path), mutation_value=value,
                            passed=True, rejection=str(exc)))
    else:
        raise AssertionError("receipt accepted mutation: " + name)
record = dict(python_optimization=sys.flags.optimize, assertions=["output-match", "stable-tip", "clean-checkout",
                          "command-exit", "empty-stderr"],
              positive_controls=1, negative_controls=len(mutations), cases=results)
(here / ("receipt-controls-optimized.json" if sys.flags.optimize else "receipt-controls.json")).write_text(json.dumps(record, indent=2) + "\n")
print(json.dumps(record, indent=2))

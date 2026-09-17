"""Record this allocation's raw-log-verified gates and exact acceptance output."""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
repo = here.parents[4]
spec = importlib.util.spec_from_file_location("verify_acceptance", here / "verify_acceptance.py")
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)
verifier.TRACKED = set(subprocess.check_output(["git", "ls-files"], cwd=repo, text=True).splitlines())
lanes = []
for name in ("prerequisites", "core-default", "core-feature", "tui"):
    row, text = verifier.lane(here / "gates-01", name)
    assert row["base"] == "8317d81a49aac1246120e806b7e380c13c738862"
    assert row["exit"] == 0 and not row["failure_lines"]
    assert row["environment"]["NEXTEST_TEST_THREADS"] == "4"
    if name != "prerequisites":
        assert row["run"] == row["passed"] > 0
    lanes.append(row)
receipt = dict(lanes=lanes, executions=sum(row["run"] for row in lanes[1:]),
               failure_names=[], unique_tests_claimed=False,
               qualification="Fresh guarded Rust filters; not independent functional acceptance.")
(here / "test-results.json").write_text(json.dumps(receipt, indent=2) + "\n")
for path in list(here.glob("*.py")) + [here.parent / "acct-fitness-76/finalize.py"]:
    ast.parse(path.read_text(), filename=str(path))
result = subprocess.run([sys.executable, "-B", str(here / "verify_acceptance.py")],
                        cwd=repo, capture_output=True, text=True)
assert result.returncode == 2 and not result.stderr
assert result.stdout == (here / "clean-checkout.stdout.txt").read_text()
(here / "acceptance-output.txt").write_text(result.stdout)
print(result.stdout, end="")
print("Fresh round-79 gates: " + json.dumps({
    "executions": receipt["executions"],
    "lanes": {row["lane"]: row["run"] for row in lanes[1:]},
    "failure_names": [],
}))

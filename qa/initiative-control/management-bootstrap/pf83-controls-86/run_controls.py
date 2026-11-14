"""Run fixed controls; --weaken deliberately removes one required observation."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-f09gap-83"))
import check_contract
import test_contract

parser = argparse.ArgumentParser()
parser.add_argument("--weaken", action="store_true")
args = parser.parse_args()
if args.weaken:
    rule = next(rule["then"] for rule in check_contract.CAPTURE["allOf"]
                if rule["if"]["properties"]["case"]["const"] == "F09")
    rule["properties"]["pre_state"]["required"].remove("captured_authority")
    print("DELIBERATE IN-MEMORY SCHEMA MUTATION: remove F09 pre_state.required captured_authority",
          flush=True)
    print("Weakened schema SHA-256 (sorted compact JSON): " + hashlib.sha256(
        json.dumps(check_contract.CAPTURE, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest(), flush=True)
    print("Fixtures unchanged; expected outcome: controls FAIL with missing captured_authority accepted.",
          flush=True)
suite = unittest.defaultTestLoader.loadTestsFromModule(test_contract)
result = unittest.TextTestRunner(verbosity=2).run(suite)
raise SystemExit(not result.wasSuccessful())

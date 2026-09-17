"""Run frozen controls, optionally weakening one in-memory schema requirement."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import unittest

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-f09gap-83"))
import check_contract as contract
import test_contract
import test_strength

parser = argparse.ArgumentParser()
parser.add_argument("--weaken", choices=(
    "capture-required", "capture-range", "capture-enum",
    "results-required", "results-range", "results-enum"))
args = parser.parse_args()
if args.weaken:
    if args.weaken == "capture-required":
        rule = next(r["then"] for r in contract.CAPTURE["allOf"]
                    if r["if"]["properties"]["case"]["const"] == "F09")
        rule["properties"]["pre_state"]["required"].remove("captured_authority")
    elif args.weaken == "capture-range":
        contract.CAPTURE["$defs"]["authority_observation"]["properties"]["observer_sequence"]["minimum"] = 0
    elif args.weaken == "capture-enum":
        contract.CAPTURE["$defs"]["authority_observation"]["properties"]["claimed_effective"]["enum"].append("unknown")
    elif args.weaken == "results-required":
        contract.RESULTS["$defs"]["result"]["required"].remove("last_checkpoint")
    elif args.weaken == "results-range":
        contract.RESULTS["$defs"]["result"]["properties"]["last_checkpoint"]["minLength"] = 0
    else:
        contract.RESULTS["$defs"]["result"]["properties"]["missing_artifacts"]["items"]["enum"].append("unknown")
    contract.meta_validate()
    print("DELIBERATE IN-MEMORY MUTATION: " + args.weaken, flush=True)
    print("Both mutated schemas meta-validate; unchanged fixtures must turn controls red.", flush=True)
    print("Mutated schema pair SHA-256: " + hashlib.sha256(json.dumps(
        [contract.CAPTURE, contract.RESULTS], sort_keys=True).encode()).hexdigest(), flush=True)
suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromModule(module)
                           for module in (test_contract, test_strength))
result = unittest.TextTestRunner(verbosity=2).run(suite)
raise SystemExit(not result.wasSuccessful())

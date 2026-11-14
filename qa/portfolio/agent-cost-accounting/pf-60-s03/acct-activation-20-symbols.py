"""Discriminate default and developer-feature native artifacts without source inspection."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=Path)
parser.add_argument("mode", choices=["default", "enabled"])
args = parser.parse_args()
data = subprocess.run(["nm", "-a", str(args.binary)], capture_output=True, text=True, check=True)
activation = [s for s in data.stdout.splitlines() if "developer_accounting_mode" in s]
collector = [s for s in data.stdout.splitlines() if "accounting" in s and "start_request" in s]
result = {
    "mode": args.mode,
    "binary": str(args.binary.resolve()),
    "sha256": hashlib.file_digest(args.binary.open("rb"), "sha256").hexdigest(),
    "activation_symbol_count": len(activation),
    "collector_control_symbol_count": len(collector),
    "activation_symbols": activation,
    "control_sample": collector[:2],
    "limitations": "Tests these exact unstripped native artifacts only. Does not prove other targets, optimized/stripped builds, release packaging, feature-unified/all-features builds or absence of differently named equivalent code. Existing collector/install APIs remain compiled in by design.",
}
print(json.dumps(result, indent=2))
assert collector, "No accounting collector control symbol: scan is not discriminating"
assert bool(activation) == (args.mode == "enabled"), "Activation symbol does not match build mode"

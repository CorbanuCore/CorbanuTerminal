"""Read-only evidence validation, including honest missing-package reporting.

Supersedes the original producer-tree finalizer. Does not rewrite historical
test-results.json or inventories. The shared verifier checks present committed
evidence and reports ignored package absence; --local-package also hashes any
local package files. Exit 2 means incomplete evidence, not a passed package.
"""
from pathlib import Path
import runpy
import sys

verifier = Path(__file__).resolve().parent.parent / "acct-inventory-79/verify_acceptance.py"
sys.argv = [str(verifier), "--local-package"]
runpy.run_path(str(verifier), run_name="__main__")

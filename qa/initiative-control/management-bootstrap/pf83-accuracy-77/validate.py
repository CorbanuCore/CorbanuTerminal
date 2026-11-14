"""Reuse the isolated gate runner with new, write-once round-77 evidence."""
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-failclosed-73"))
import validate

validate.HERE = HERE
if __name__ == "__main__":
    raise SystemExit(validate.main())

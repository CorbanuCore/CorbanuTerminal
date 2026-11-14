"""Hash every file and record every link in an explicitly supplied package."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--package", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
package = args.package.resolve(strict=True)
rows = []
for path in sorted(package.rglob("*")):
    row = {"path": str(path.relative_to(package))}
    if path.is_symlink():
        resolved = path.resolve(strict=True)
        assert resolved.is_relative_to(package), "package link escapes package"
        row["link"] = str(path.readlink())
    elif path.is_file():
        with path.open("rb") as source:
            row.update(size=path.stat().st_size, sha256=hashlib.file_digest(source, "sha256").hexdigest())
    else:
        continue
    rows.append(row)
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps({"package": str(package), "files": rows}, indent=2) + "\n")
print(f"Inventoried {len(rows)} package files/links")

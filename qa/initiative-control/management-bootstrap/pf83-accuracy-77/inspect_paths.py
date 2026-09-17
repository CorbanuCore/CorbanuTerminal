"""Read only build-path strings in the frozen binaries; never execute them."""
import json
import mmap
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-handoff-70"))
from verify_bundle import sha

root = HERE.parent / "pf83-handoff-70/artifacts/handoff-root/package"
rows = []
for path in sorted(root.iterdir()):
    with path.open("rb") as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as raw:
        paths = sorted(set(match.group().decode("ascii") for match in re.finditer(
            rb"/(?:Volumes|Users|private/tmp)/[A-Za-z0-9_./+@-]+", raw)))
    selectors = {
        "account_toolchain": "/Users/",
        "internal_source_filename": "/source/codex-rs/",
        "dependency_name_version": "/registry/src/",
        "build_object": "/release/deps/",
    }
    examples = {name: next((value for value in paths if fragment in value), None)
                for name, fragment in selectors.items()}
    rows.append(dict(binary=path.name, sha256=sha(path),
                     distinct_matching_path_strings=len(paths), examples=examples))
print(json.dumps(dict(
    method="Read-only ASCII absolute build-path regex scan, not a secrets audit or complete metadata inventory",
    binary_execution=False, files=rows), indent=2, sort_keys=True))

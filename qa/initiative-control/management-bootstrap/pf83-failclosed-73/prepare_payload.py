"""Future local preparation: allowlisted actor assets, no provenance or runtime."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pf83-handoff-70"))
from verify_bundle import require, sha, verify

ATTESTATION = "9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27"
ORIGINAL = "c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726"


def selected(bundle):
    identity = verify(bundle, ATTESTATION)
    files = { "package/" + name: bundle / "package" / name for name in identity["files"] }
    for name, expected in {
        "original-F01-F11.md": ORIGINAL,
        "navigation.md": sha(HERE.parent / "pf83-handoff-70" / "navigation.md"),
    }.items():
        path = bundle / "packet" / name
        require(not path.is_symlink() and sha(path) == expected, "packet asset mismatch")
        files["packet/" + name] = path
    return files


def prepare(bundle, output):
    files = selected(bundle)
    output.mkdir(mode=0o700)
    inventory = {name: dict(sha256=sha(path), mode=0o555 if name.startswith("package/") else 0o444)
                 for name, path in sorted(files.items())}
    raw = (json.dumps(inventory, indent=2, sort_keys=True) + "\n").encode()
    archive = output / "actor-assets.tar"
    with tarfile.open(archive, "x", format=tarfile.USTAR_FORMAT) as tar:
        for name, path in sorted(files.items()):
            info = tarfile.TarInfo(name)
            info.size = path.stat().st_size
            info.mode = inventory[name]["mode"]
            with path.open("rb") as stream:
                tar.addfile(info, stream)
        info = tarfile.TarInfo("stage-inventory.json")
        info.size, info.mode = len(raw), 0o444
        tar.addfile(info, io.BytesIO(raw))
    receipt = dict(payload_kind="actor-assets-only-v1", archive_sha256=sha(archive),
                   inventory_sha256=hashlib.sha256(raw).hexdigest(),
                   attestation_sha256=ATTESTATION, files=inventory,
                   provenance_location="host-only; original bundle unchanged")
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    prepare(args.bundle, args.output)

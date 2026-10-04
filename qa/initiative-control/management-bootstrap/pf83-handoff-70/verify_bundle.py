"""Read-only relocatable bundle identity check. Never launch a packaged binary."""
import argparse
import hashlib
import json
from pathlib import Path
import stat


def sha(path):
    with path.open("rb") as stream:
        digest = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
        return digest.hexdigest()


def require(ok, message):
    if not ok:
        raise ValueError(message)


def member(root, name, directory=False):
    relative = Path(name)
    require(not relative.is_absolute() and relative.parts
            and all(p not in (".", "..") for p in relative.parts), "unsafe relative path")
    path = root
    for part in relative.parts:
        path = path / part
        require(not path.is_symlink(), "symlink member")
    mode = path.stat().st_mode
    require(stat.S_ISDIR(mode) if directory else stat.S_ISREG(mode), "special/missing member")
    require(path.resolve().is_relative_to(root.resolve()), "member escapes declared root")
    return path


def verify(root, expected):
    require(not root.is_symlink() and root.is_dir(), "bundle root must be a directory")
    att_path = member(root, "build-attestation.json")
    require(sha(att_path) == expected, "attestation digest mismatch")
    att = json.loads(att_path.read_bytes())
    require(att["schema_version"] == 2 and att["root"] == "attestation-parent",
            "unknown root convention")
    provenance_path = member(root, att["provenance"]["path"])
    require(sha(provenance_path) == att["provenance"]["sha256"], "provenance mismatch")
    provenance = json.loads(provenance_path.read_bytes())
    require(provenance["exit_code"] == 0 and provenance["source_before"]["status"] == ""
            and provenance["source_before"] == provenance["source_after"],
            "build not successful/clean")
    require(att["source_before"] == provenance["source_before"]
            and att["source_after"] == provenance["source_after"], "source mismatch")
    pin = att["package"]
    manifest_path = member(root, pin["manifest"])
    require(sha(manifest_path) == pin["manifest_sha256"]
            == provenance["package"]["manifest_sha256"], "manifest digest mismatch")
    manifest = json.loads(manifest_path.read_bytes())
    require(manifest["commit"] == att["source_before"]["commit"]
            and manifest["tree"] == att["source_before"]["tree"], "source pin mismatch")
    package = member(root, pin["path"], directory=True)
    names = {"corbanu", "corbanu-acp", "corbanu-walletd", "codex-code-mode-host"}
    require(set(manifest["files"]) == names == {p.name for p in package.iterdir()},
            "package inventory mismatch")
    require(stat.S_IMODE(package.stat().st_mode) == 0o555, "package directory mode")
    for name, info in manifest["files"].items():
        path = member(package, name)
        require(sha(path) == info["sha256"], "binary digest mismatch: " + name)
        require(stat.S_IMODE(path.stat().st_mode) == info["mode"] == 0o555,
                "binary mode mismatch: " + name)
    require(manifest["entrypoint"] == "corbanu"
            and att["binary"]["path"] == pin["path"] + "/corbanu"
            and att["binary"]["sha256"] == manifest["files"]["corbanu"]["sha256"]
            == provenance["binary"]["sha256"], "entrypoint mismatch")
    return dict(attestation_sha256=expected, manifest_sha256=sha(manifest_path),
                source_commit=manifest["commit"], source_tree=manifest["tree"],
                files=manifest["files"], verified_binaries=len(names),
                packaged_binary_executed=False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("attestation_sha256")
    args = parser.parse_args()
    print(json.dumps(verify(args.root, args.attestation_sha256), indent=2, sort_keys=True))

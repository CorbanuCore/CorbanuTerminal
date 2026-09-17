"""Additive host-only relocation proof; no guest contact or packaged execution."""
import json
from pathlib import Path
import shutil
import subprocess

from verify_bundle import sha, verify

HERE = Path(__file__).resolve().parent
OLD = HERE.parent / "pf83-package-67"


def write(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, sort_keys=True)
        stream.write("\n")


def main():
    artifacts = HERE / "artifacts"
    artifacts.mkdir(mode=0o700)
    original = OLD / "build-attestation.json"
    assert sha(original) == "349ecadeae3d1d57125acc012103aa6bb92acf1eec42678669a817aaec162d53"
    assert sha(original) == sha(OLD / "artifacts/build-attestation.json")
    att = json.loads(original.read_bytes())
    # Historical build paths remain in immutable provenance, never used as locators.
    for key in ("cwd", "environment", "script", "recipe_origin", "toolchain",
                "cache_seed", "cache_method", "argv", "shell_command", "started_ns",
                "finished_ns", "offline", "verification", "exit_code"):
        del att[key]
    att.update(schema_version=2, root="attestation-parent",
               provenance=dict(path="build-provenance.json", sha256=sha(original)),
               relocation="Move the entire bundle; resolve package/manifest/provenance relative to the attestation parent. Historical provenance paths are observations, not locators.",
               cache_limit="The seed was bound by path only at build time, not by content. This attestation cannot exclude cache reuse of an artifact built from another source pin or prove a clean/reproducible build. Post-build hashing cannot reconstruct that missing evidence. Source state, successful build command and final binary hashes remain evidenced; cache-origin exclusion is unproven.")
    att["package"]["path"] = "package"
    att["package"]["manifest"] = "package-manifest.json"
    att["binary"]["path"] = "package/corbanu"
    att["inspection_after_move"] = [
        "Compare exact attestation bytes to the independently frozen attestation SHA-256; do not edit or repin it.",
        "Rehash provenance and manifest bytes, validate source commit/tree and clean successful build record.",
        "Enumerate the complete package, reject symlinks/special files/extras, recompute each binary SHA-256 and mode.",
        "If creating a new transport archive, hash and record that archive separately; its digest is not the attestation digest.",
        "Repeat after extraction and before/after each case; filesystem modes are identity checks, not sandbox enforcement."
    ]
    source = artifacts / "before-move"
    source.mkdir()
    subprocess.run(["/bin/cp", "-cpR", str(OLD / "artifacts/package"), str(source / "package")], check=True)
    shutil.copyfile(OLD / "artifacts/package-manifest.json", source / "package-manifest.json")
    shutil.copyfile(original, source / "build-provenance.json")
    write(source / "build-attestation.json", att)
    digest = sha(source / "build-attestation.json")
    before = verify(source, digest)
    destination = artifacts / "handoff-root"
    source.rename(destination)
    after = verify(destination, digest)
    assert before == after and not source.exists()
    for name in ("build-attestation.json", "build-provenance.json", "package-manifest.json"):
        shutil.copyfile(destination / name, HERE / name)
    write(HERE / "move-proof.json", dict(operation="rename", before_root=str(source),
        after_root=str(destination), old_root_absent=not source.exists(), before=before,
        after=after, same_attestation_bytes=True, guest_contact=False))
    (HERE / "attestation.sha256").write_text(digest + "  build-attestation.json\n")
    print(json.dumps(after))


if __name__ == "__main__":
    main()

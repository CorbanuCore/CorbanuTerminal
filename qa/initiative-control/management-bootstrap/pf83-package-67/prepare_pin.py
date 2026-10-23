"""Prepare, but never apply, the out-of-scope live harness pin correction."""
import difflib
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ART = HERE / "artifacts"
FIXTURES = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/fixtures.py")


def main():
    original = FIXTURES.read_text()
    manifest_path = ART / "package-manifest.json"
    raw = manifest_path.read_bytes()
    manifest = json.loads(raw)
    digest = hashlib.sha256(raw).hexdigest()
    old = ('CANDIDATE = dict(commit="e3bd579bf4e0c7c58ad863af2a9c6098e2297f98",\n'
           '                 tree="dd4ea6fc1584a10064aa15b38f50bd7eae35c259",')
    new = (f'CANDIDATE = dict(commit="{manifest["commit"]}",\n'
           f'                 tree="{manifest["tree"]}",')
    assert original.count(old) == 1
    proposed = original.replace(old, new)
    patch = "".join(difflib.unified_diff(original.splitlines(keepends=True),
                      proposed.splitlines(keepends=True), fromfile="a/fixtures.py", tofile="b/fixtures.py"))
    (HERE / "harness-candidate-pin.patch").write_text(patch)
    # Evaluate only the existing verifier API; neither module's CLI is called.
    def namespace(code):
        scope = dict(__file__=str(FIXTURES), __name__="pf83_pin_verification")
        exec(compile(code, str(FIXTURES), "exec"), scope)
        return scope
    original_ns = namespace(original)
    try:
        original_ns["verify_package"](ART / "package", manifest_path, digest)
    except Exception as error:
        assert str(error) == "source pin mismatch", str(error)
        old_verdict = str(error)
    else:
        raise AssertionError("stale live pin unexpectedly accepted new package")
    proposed_ns = namespace(proposed)
    assert proposed_ns["verify_package"](ART / "package", manifest_path, digest) == manifest
    assert FIXTURES.read_text() == original, "live harness unexpectedly modified"
    record = dict(commit=manifest["commit"], tree=manifest["tree"],
                  package=str(ART / "package"), manifest=str(manifest_path),
                  manifest_sha256=digest, archive_sha256=None,
                  live_harness_updated=False, live_harness_verdict=old_verdict,
                  proposed_patch_verdict="accepted by unchanged verify_package with proposed source pin",
                  original_fixtures_sha256=hashlib.sha256(original.encode()).hexdigest(),
                  proposed_fixtures_sha256=hashlib.sha256(proposed.encode()).hexdigest(),
                  patch_sha256=hashlib.sha256(patch.encode()).hexdigest())
    (HERE / "candidate-pin.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record))


if __name__ == "__main__":
    main()

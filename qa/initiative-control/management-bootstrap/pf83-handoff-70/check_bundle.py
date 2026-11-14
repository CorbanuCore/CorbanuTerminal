"""Exercise relocation verifier rejection paths on disposable local copies."""
import json
from pathlib import Path
import subprocess
import tempfile

from verify_bundle import sha, verify

HERE = Path(__file__).resolve().parent
SOURCE = HERE / "artifacts/handoff-root"
DIGEST = "9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27"


def main():
    outcomes = {}
    for scenario in ("changed-attestation", "changed-manifest", "changed-binary",
                     "extra-file", "symlink", "wrong-mode", "absolute-path", "parent-path"):
        with tempfile.TemporaryDirectory(dir=HERE / "artifacts") as temp:
            root = Path(temp) / "bundle"
            subprocess.run(["/bin/cp", "-cpR", str(SOURCE), str(root)], check=True)
            package = root / "package"
            package.chmod(0o755)
            expected = DIGEST
            if scenario in ("changed-attestation", "changed-manifest", "changed-binary"):
                relative = {"changed-attestation": "build-attestation.json",
                            "changed-manifest": "package-manifest.json",
                            "changed-binary": "package/corbanu"}[scenario]
                path = root / relative
                path.chmod(0o644)
                with path.open("r+b") as stream:
                    stream.write(b"X")
                if scenario == "changed-binary":
                    path.chmod(0o555)
            elif scenario == "extra-file":
                (package / "extra").write_bytes(b"synthetic")
            elif scenario == "symlink":
                (package / "corbanu").unlink()
                (package / "corbanu").symlink_to(SOURCE / "package/corbanu")
            elif scenario == "wrong-mode":
                (package / "corbanu").chmod(0o444)
            else:
                path = root / "build-attestation.json"
                att = json.loads(path.read_bytes())
                att["package"]["path"] = str(SOURCE / "package") if scenario == "absolute-path" else "../package"
                path.chmod(0o644)
                path.write_text(json.dumps(att))
                expected = sha(path)
            package.chmod(0o555)
            try:
                verify(root, expected)
            except ValueError as error:
                outcomes[scenario] = dict(rejected=True, reason=str(error))
            else:
                raise AssertionError("accepted tampering: " + scenario)
            finally:
                package.chmod(0o755)
    identity = verify(SOURCE, DIGEST)
    record = dict(negative_cases=outcomes, rejected=len(outcomes),
                  original_still_verified=identity, packaged_execution=False)
    (HERE / "verifier-checks.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    print(json.dumps(record))


if __name__ == "__main__":
    main()

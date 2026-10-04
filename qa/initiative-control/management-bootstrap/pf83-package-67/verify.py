"""Read-only source/package checks, writing only this allocation's evidence."""
import ast
import csv
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
ART = HERE / "artifacts"
HARNESS = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915")


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    freeze = HERE.parent / "pf83-exit-61-production-freeze.md"
    rows = re.findall(r"\| `(codex-rs/[^`]+)` \| `([a-f0-9]{64})`", freeze.read_text())
    assert len(rows) == 34
    comparison = [dict(path=p, frozen_sha256=h, checkout_sha256=sha(REPO / p),
                       built_source_sha256=sha(ART / "source" / p)) for p, h in rows]
    for row in comparison:
        row["matches"] = row["frozen_sha256"] == row["checkout_sha256"] == row["built_source_sha256"]
    (HERE / "production-comparison.json").write_text(json.dumps(comparison, indent=2) + "\n")
    assert all(row["matches"] for row in comparison)
    fixture_code = (HARNESS / "fixtures.py").read_text()
    parsed = ast.parse(fixture_code)
    mapping = next(ast.literal_eval(node.value) for node in parsed.body if isinstance(node, ast.Assign)
                   and any(isinstance(t, ast.Name) and t.id == "MAP" for t in node.targets))
    with (HERE.parent / "pf83-packaged-63-case-inputs.tsv").open() as stream:
        packets = list(csv.DictReader(stream, delimiter="\t"))
    assert len(packets) == 288
    counts = {}
    for row in packets:
        path = HARNESS / row["harness_relative_packet_path"]
        assert sha(path) == row["sha256"]
        group = path.name.split("--")[0]
        cases = sorted(case for case, prefixes in mapping.items()
                       if any(group == prefix or group.startswith(prefix + "-") for prefix in prefixes))
        assert cases == row["all_mapped_cases"].split(","), (path.name, cases, row)
        assert row["projected_case"] in cases
        counts[row["projected_case"]] = counts.get(row["projected_case"], 0) + 1
    manifest_path = ART / "package-manifest.json"
    attestation = json.loads((ART / "build-attestation.json").read_text())
    manifest = json.loads(manifest_path.read_text())
    assert attestation["source_before"] == attestation["source_after"]
    assert attestation["source_before"]["status"] == ""
    assert attestation["exit_code"] == 0
    assert manifest["commit"] == attestation["source_before"]["commit"]
    assert manifest["tree"] == attestation["source_before"]["tree"]
    assert sha(manifest_path) == attestation["package"]["manifest_sha256"]
    binaries = {"corbanu", "corbanu-acp", "corbanu-walletd", "codex-code-mode-host"}
    package = ART / "package"
    assert {p.name for p in package.iterdir()} == set(manifest["files"]) == binaries
    assert package.stat().st_mode & 0o777 == 0o555
    for name in binaries:
        path = package / name
        assert path.is_file() and not path.is_symlink()
        assert sha(path) == manifest["files"][name]["sha256"]
        assert path.stat().st_mode & 0o777 == manifest["files"][name]["mode"] == 0o555
    inspections = {}
    for name in sorted(binaries):
        path = package / name
        inspections[name] = {}
        for argv in (["/usr/bin/file", str(path)], ["/usr/bin/otool", "-L", str(path)]):
            result = subprocess.run(argv, text=True, capture_output=True)
            assert result.returncode == 0
            inspections[name][argv[0]] = dict(argv=argv, exit_code=result.returncode, stdout=result.stdout)
    (HERE / "binary-inspection.json").write_text(json.dumps(inspections, indent=2) + "\n")
    receipt = dict(production_files=34, production_matches=34, packet_paths=288,
                   packet_counts=counts, package_manifest_sha256=sha(manifest_path),
                   build_attestation_sha256=sha(ART / "build-attestation.json"),
                   executable_inventory=sorted(binaries), no_packaged_execution=True,
                   live_harness_pin_updated=False)
    (HERE / "verification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    main()

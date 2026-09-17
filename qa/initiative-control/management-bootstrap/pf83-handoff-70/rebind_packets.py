"""Regenerate the selected 288 coordinator packets additively; never alter originals."""
import argparse
import csv
import io
import json
from pathlib import Path

from verify_bundle import sha, verify

HERE = Path(__file__).resolve().parent
HARNESS = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915")
INDEX = HERE.parent / "pf83-packaged-63-case-inputs.tsv"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("attestation_sha256")
    args = parser.parse_args()
    identity = verify(args.bundle, args.attestation_sha256)
    with INDEX.open() as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    assert len(rows) == 288
    assert len({r["harness_relative_packet_path"] for r in rows}) == 288
    args.output.mkdir(mode=0o700)
    packets = args.output / "packets"
    packets.mkdir()
    candidate = dict(commit=identity["source_commit"], tree=identity["source_tree"],
                     version=None, package_sha256=identity["manifest_sha256"],
                     archive_sha256=None,
                     attestation_sha256=identity["attestation_sha256"],
                     version_status="not executed; identity is the manifest and binary digests",
                     archive_status="transport digest bound separately in staging receipt")
    counts = {}
    output = io.StringIO()
    writer = csv.writer(output, delimiter="\t", lineterminator="\n")
    writer.writerow(["projected_case", "source_sha256", "sha256", "packet_path", "all_mapped_cases"])
    for row in rows:
        original = HARNESS / row["harness_relative_packet_path"]
        assert sha(original) == row["sha256"]
        prior = json.loads(original.read_bytes())
        assert prior["candidate"]["commit"] == "e3bd579bf4e0c7c58ad863af2a9c6098e2297f98"
        assert prior["candidate"]["tree"] == "dd4ea6fc1584a10064aa15b38f50bd7eae35c259"
        revised = dict(prior, candidate=candidate)
        destination = packets / original.name
        with destination.open("x") as stream:
            json.dump(revised, stream, indent=2, sort_keys=True, ensure_ascii=False, allow_nan=False)
            stream.write("\n")
        check = json.loads(destination.read_bytes())
        assert check.pop("candidate") == candidate
        assert check == {k: v for k, v in prior.items() if k != "candidate"}
        assert sha(original) == row["sha256"]
        writer.writerow([row["projected_case"], row["sha256"], sha(destination),
                         "packets/" + original.name, row["all_mapped_cases"]])
        counts[row["projected_case"]] = counts.get(row["projected_case"], 0) + 1
    (args.output / "packet-bindings.tsv").write_text(output.getvalue())
    record = dict(counts=counts, total=288, binding=candidate,
                  source_index_sha256=sha(INDEX),
                  packet_bindings_sha256=sha(args.output / "packet-bindings.tsv"),
                  changed_fields=["candidate"], originals_unchanged=True,
                  executor_visible=False, executed_cases=0)
    (args.output / "packet-rebinding.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    print(json.dumps(record))


if __name__ == "__main__":
    main()

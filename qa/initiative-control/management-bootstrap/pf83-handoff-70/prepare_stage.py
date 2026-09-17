"""Create and locally extract a frozen transport archive. Never use SSH or run a case."""
import json
from pathlib import Path
import shutil
import stat
import tarfile

from verify_bundle import sha, verify

HERE = Path(__file__).resolve().parent
HARNESS = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915")
DIGEST = "9d33fcc21788a84c61011fe64a3607fc7569b390a260e5c19a7b9d3e6582ce27"


def main():
    bundle = HERE / "artifacts/handoff-root"
    verify(bundle, DIGEST)
    # Additive staging assets; attestation/package/manifest bytes remain unchanged.
    runtime = bundle / "runtime"
    runtime.mkdir()
    for name in ("native_guest.py", "macos_preflight.py", "macos_boundary.py",
                 "macos_bridge.sbpl", "native_prompt_guard.py"):
        path = HARNESS / name
        assert path.is_file() and not path.is_symlink()
        shutil.copyfile(path, runtime / name)
    shutil.copyfile(HERE / "verify_bundle.py", runtime / "verify_bundle.py")
    packet = bundle / "packet"
    packet.mkdir()
    original = HARNESS / "normalized/original-F01-F11.md"
    assert sha(original) == "c97bf701cd7aff2e26f08884f35dbc9a4fc33b1eef14eee6328d2ea1f1411726"
    shutil.copyfile(original, packet / "original-F01-F11.md")
    shutil.copyfile(HERE / "navigation.md", packet / "navigation.md")
    inventory = {}
    for path in sorted(bundle.rglob("*")):
        assert not path.is_symlink()
        if path.is_file():
            mode = 0o555 if path.parent.name == "package" else 0o444
            path.chmod(mode)
            inventory[path.relative_to(bundle).as_posix()] = dict(sha256=sha(path), mode=mode)
        else:
            assert path.is_dir()
    inventory_path = bundle / "stage-inventory.json"
    inventory_path.write_text(json.dumps(inventory, indent=2, sort_keys=True) + "\n")
    inventory_path.chmod(0o444)
    archive = HERE / "artifacts/pf83-handoff-70.tar"
    with tarfile.open(archive, "x", format=tarfile.USTAR_FORMAT) as tar:
        for path in sorted(bundle.rglob("*")):
            name = path.relative_to(bundle).as_posix()
            item = tar.gettarinfo(str(path), arcname=name)
            item.uid = item.gid = 0
            item.uname = item.gname = ""
            item.mtime = 0
            if path.is_file():
                with path.open("rb") as stream:
                    tar.addfile(item, stream)
            else:
                tar.addfile(item)
    extracted = HERE / "artifacts/extracted"
    extracted.mkdir()
    with tarfile.open(archive) as tar:
        for item in tar.getmembers():
            assert not item.name.startswith("/") and ".." not in Path(item.name).parts
            assert item.isfile() or item.isdir()
        # Only this locally authored, checked regular-file/directory archive.
        tar.extractall(extracted, filter="fully_trusted")
    after = verify(extracted, DIGEST)
    assert sha(extracted / "stage-inventory.json") == sha(inventory_path)
    actual = {}
    for path in sorted(extracted.rglob("*")):
        assert not path.is_symlink()
        if path.is_file() and path.name != "stage-inventory.json":
            actual[path.relative_to(extracted).as_posix()] = dict(
                sha256=sha(path), mode=stat.S_IMODE(path.stat().st_mode))
    assert actual == inventory
    shutil.copyfile(inventory_path, HERE / "stage-inventory.json")
    receipt = dict(archive_path="artifacts/pf83-handoff-70.tar", archive_sha256=sha(archive),
                   stage_inventory_sha256=sha(inventory_path), attestation_sha256=DIGEST,
                   archive_members=len(inventory) + 1, extracted_identity=after,
                   local_extraction_verified=True, guest_contact=False, executed_cases=0,
                   packet_assets=["original-F01-F11.md", "navigation.md"],
                   coordinator_packets_staged=False, runtime_case_dispatch=False)
    (HERE / "stage-receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    main()

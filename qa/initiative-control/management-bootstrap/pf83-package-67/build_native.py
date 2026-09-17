"""Scope-confined successor to harness increment-26's pinned-source build."""
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
HARNESS = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915")
BUILD = HERE / "artifacts"
SOURCE = BUILD / "source"
RUST = Path("/Users/Neo/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin")
PIN = dict(commit="ae5981d2d7d762dfd6d54e0ac0967729bf88f082",
           tree="834f57388066d33c27fdee6e3b234dec7fa0b0e6", status="")
BINS = ("corbanu", "corbanu-acp", "corbanu-walletd", "codex-code-mode-host")


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def state():
    def git(*args):
        return subprocess.check_output(["git", "-C", str(SOURCE), *args], text=True).strip()
    return dict(commit=git("rev-parse", "HEAD"), tree=git("rev-parse", "HEAD^{tree}"),
                status=git("status", "--porcelain=v1", "--untracked-files=all"))


def main():
    os.umask(0o077)
    BUILD.mkdir()
    journal = []
    def run(argv):
        result = subprocess.run(argv, check=False)
        journal.append(dict(argv=argv, shell_command=shlex.join(argv), exit_code=result.returncode))
        (BUILD / "preparation-commands.json").write_text(json.dumps(journal, indent=2) + "\n")
        result.check_returncode()
    run(["git", "clone", "--shared", "--no-checkout", str(REPO), str(SOURCE)])
    run(["git", "-C", str(SOURCE), "checkout", "--detach", PIN["commit"]])
    assert state() == PIN
    for name in ("build-home", "tmp", "cargo-home", "cache"):
        (BUILD / name).mkdir()
    seed = HARNESS / "increment-27/build"
    for name in ("registry", "git"):
        run(["/bin/cp", "-cR", str(seed / "cargo-home" / name), str(BUILD / "cargo-home" / name)])
    run(["/bin/cp", "-cR", str(seed / "target"), str(BUILD / "target")])
    env = dict(PATH=str(RUST) + ":/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
               HOME=str(BUILD / "build-home"), TMPDIR=str(BUILD / "tmp"),
               CARGO_HOME=str(BUILD / "cargo-home"), CARGO_TARGET_DIR=str(BUILD / "target"),
               XDG_CACHE_HOME=str(BUILD / "cache"))
    tools = {name: dict(path=str(RUST / name), sha256=sha(RUST / name),
             version=subprocess.check_output([str(RUST / name), "-vV"], env=env, text=True).strip())
             for name in ("cargo", "rustc")}
    before = state()
    assert before == PIN
    argv = [str(RUST / "cargo"), "build", "--release", "--locked"]
    for name in BINS:
        argv += ["--bin", name]
    record = dict(argv=argv, shell_command=shlex.join(argv), cwd=str(SOURCE / "codex-rs"),
                  environment=env, source_before=before, toolchain=tools, started_ns=time.time_ns(),
                  offline=False, script=dict(path=str(Path(__file__)), sha256=sha(Path(__file__))),
                  cache_seed=str(seed), cache_method="APFS clone copies; no profiles or credentials",
                  recipe_origin=dict(path=str(HARNESS / "increment-26/build_native.py"),
                                     sha256=sha(HARNESS / "increment-26/build_native.py")))
    attestation = BUILD / "build-attestation.json"
    def save():
        attestation.write_text(json.dumps(record, indent=2) + "\n")
    save()
    with (BUILD / "build.stdout").open("xb") as out, (BUILD / "build.stderr").open("xb") as err:
        result = subprocess.run(argv, cwd=SOURCE / "codex-rs", env=env, stdout=out, stderr=err)
    record.update(exit_code=result.returncode, finished_ns=time.time_ns(), source_after=state())
    save()
    assert record["source_after"] == PIN, "source changed during build"
    if result.returncode:
        return result.returncode
    package = BUILD / "package"
    package.mkdir()
    for name in BINS:
        shutil.copy2(BUILD / "target/release" / name, package / name)
        (package / name).chmod(0o555)
    package.chmod(0o555)
    manifest = dict(commit=PIN["commit"], tree=PIN["tree"], entrypoint="corbanu",
                    format="cargo-darwin-runtime-binaries",
                    limitations=["Unsigned local qualification package, not a notarized release archive.",
                                 "Uses guest system shell/search tools; optional bundled rg and patched zsh are not included."],
                    files={name: dict(sha256=sha(package / name), mode=(package / name).stat().st_mode & 0o777)
                           for name in sorted(BINS)})
    path = BUILD / "package-manifest.json"
    path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
    path.chmod(0o444)
    assert set(p.name for p in package.iterdir()) == set(BINS)
    assert all(not (package / name).is_symlink() and (package / name).is_file()
               and sha(package / name) == info["sha256"]
               and (package / name).stat().st_mode & 0o777 == info["mode"] == 0o555
               for name, info in manifest["files"].items())
    record["binary"] = dict(path=str(package / "corbanu"), sha256=sha(package / "corbanu"))
    record["package"] = dict(path=str(package), manifest=str(path), manifest_sha256=sha(path))
    record["verification"] = "Exact four-file regular executable inventory, hashes and modes checked; live harness pin unchanged (outside write scope)."
    save()
    print(json.dumps(dict(exit_code=0, package=record["package"])))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

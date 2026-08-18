#!/usr/bin/env python3

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import textwrap
import unittest


INSTALLER = Path(__file__).with_name("install.sh")
VERSION = "9.8.7"
TARGET = "x86_64-unknown-linux-gnu"


class CorbanuReleaseContractTest(unittest.TestCase):
    def test_windows_installer_prunes_releases_after_command_verification(self) -> None:
        installer = INSTALLER.with_suffix(".ps1").read_text(encoding="utf-8")

        self.assertIn("$env:CORBANU_KEEP_RELEASES", installer)
        self.assertIn("function Remove-OldStandaloneReleases", installer)
        self.assertLess(
            installer.index(
                "Test-VisibleTerminalCommands -VisibleBinDir $visibleBinDir"
            ),
            installer.index(
                "Remove-OldStandaloneReleases -ReleasesDir $releasesDir "
                "-CurrentDir $currentDir -Keep $KeepReleases"
            ),
        )

    def test_installer_prefers_native_corbanu_release_assets(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive, checksum, metadata = create_release_fixture(
                root, asset_family="corbanu-terminal-package"
            )
            result, requests = run_installer(root, metadata, archive, checksum)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(
                any("corbanu-terminal-package_SHA256SUMS" in url for url in requests)
            )
            self.assertTrue(
                any(
                    f"corbanu-terminal-package-{TARGET}.tar.gz" in url
                    for url in requests
                )
            )
            version = subprocess.run(
                [root / "install-bin" / "corbanu", "--version"],
                capture_output=True,
                check=True,
                text=True,
            )
            self.assertEqual(version.stdout, f"corbanu {VERSION}\n")
            self.assertFalse((root / "install-bin" / "pfterminal").exists())
            self.assertNotIn("compatibility alias", result.stdout)

    def test_unrelated_asset_family_fails_with_corbanu_diagnostic(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            metadata = json.dumps(
                {
                    "tag_name": f"rust-v{VERSION}",
                    "assets": [
                        {
                            "name": "unrelated-package.tar.gz",
                            "digest": f"sha256:{'0' * 64}",
                        }
                    ],
                },
                indent=2,
            )
            result, _requests = run_installer(root, metadata)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                f"Could not find Corbanu Terminal package or compatible legacy release assets for {VERSION}.",
                result.stderr,
            )

    def test_bundled_package_installs_without_network_requests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive, checksum, metadata = create_release_fixture(root)
            result, requests = run_installer(
                root,
                metadata,
                archive,
                checksum,
                bundled=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(requests, [])
            self.assertIn("Using bundled Corbanu Terminal package", result.stdout)
            self.assertTrue((root / "install-bin" / "corbanu").is_file())


def create_release_fixture(
    root: Path, *, asset_family: str = "corbanu-terminal-package"
) -> tuple[Path, Path, str]:
    package = root / "package"
    for directory in ("bin", "codex-path", "codex-resources"):
        (package / directory).mkdir(parents=True, exist_ok=True)
    (package / "codex-package.json").write_text("{}\n", encoding="utf-8")
    write_executable(
        package / "bin" / "corbanu",
        f"#!/bin/sh\nprintf 'corbanu {VERSION}\\n'\n",
    )
    write_executable(package / "bin" / "corbanu-debug", "#!/bin/sh\nexit 0\n")
    write_executable(package / "bin" / "corbanu-acp", "#!/bin/sh\nexit 0\n")
    write_executable(package / "bin" / "corbanu-walletd", "#!/bin/sh\nexit 0\n")
    write_executable(package / "bin" / "codex-code-mode-host", "#!/bin/sh\nexit 0\n")
    write_executable(package / "codex-path" / "rg", "#!/bin/sh\nexit 0\n")
    write_executable(package / "codex-resources" / "bwrap", "#!/bin/sh\nexit 0\n")

    asset = f"{asset_family}-{TARGET}.tar.gz"
    archive = root / asset
    with tarfile.open(archive, "w:gz") as output:
        for child in package.iterdir():
            output.add(child, arcname=child.name)

    archive_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = root / f"{asset_family}_SHA256SUMS"
    checksum.write_text(f"{archive_digest}  {asset}\n", encoding="utf-8")
    checksum_digest = hashlib.sha256(checksum.read_bytes()).hexdigest()
    metadata = json.dumps(
        {
            "tag_name": f"rust-v{VERSION}",
            "assets": [
                {"name": asset, "digest": f"sha256:{archive_digest}"},
                {
                    "name": checksum.name,
                    "digest": f"sha256:{checksum_digest}",
                },
            ],
        },
        indent=2,
    )
    return archive, checksum, metadata


def run_installer(
    root: Path,
    metadata: str,
    archive: Path | None = None,
    checksum: Path | None = None,
    *,
    bundled: bool = False,
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    fake_bin = root / "fake-bin"
    fake_bin.mkdir()
    request_log = root / "requests.log"
    fake_curl = fake_bin / "curl"
    fake_curl.write_text(
        textwrap.dedent(
            """\
            #!/bin/sh
            url=""
            output=""
            previous=""
            for argument in "$@"; do
              case "$argument" in https://*) url="$argument" ;; esac
              if [ "$previous" = "-o" ]; then output="$argument"; fi
              previous="$argument"
            done
            printf '%s\n' "$url" >>"$CORBANU_TEST_REQUEST_LOG"
            case "$url" in
              https://api.github.com/*)
                printf '%s\n' "$CORBANU_TEST_METADATA"
                ;;
              */corbanu-terminal-package_SHA256SUMS)
                cp "$CORBANU_TEST_CHECKSUM" "$output"
                ;;
              */corbanu-terminal-package-*.tar.gz)
                cp "$CORBANU_TEST_ARCHIVE" "$output"
                ;;
              *) exit 22 ;;
            esac
            """
        ),
        encoding="utf-8",
    )
    fake_curl.chmod(0o755)

    home = root / "home"
    home.mkdir()
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
            "PATH": f"{fake_bin}:/usr/bin:/bin",
            "SHELL": "/bin/sh",
            "CORBANU_HOME": str(root / "corbanu-home"),
            "CORBANU_INSTALL_DIR": str(root / "install-bin"),
            "CORBANU_NON_INTERACTIVE": "1",
            "CORBANU_RELEASE": VERSION,
            "CORBANU_TEST_ARCHIVE": str(archive or ""),
            "CORBANU_TEST_CHECKSUM": str(checksum or ""),
            "CORBANU_TEST_METADATA": metadata,
            "CORBANU_TEST_REQUEST_LOG": str(request_log),
        }
    )
    if bundled:
        env["CORBANU_PACKAGE_ARCHIVE"] = str(archive or "")
        env["CORBANU_CHECKSUM_MANIFEST"] = str(checksum or "")
    result = subprocess.run(
        ["/bin/sh", INSTALLER], capture_output=True, check=False, env=env, text=True
    )
    requests = (
        request_log.read_text(encoding="utf-8").splitlines()
        if request_log.exists()
        else []
    )
    return result, requests


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


if __name__ == "__main__":
    unittest.main()

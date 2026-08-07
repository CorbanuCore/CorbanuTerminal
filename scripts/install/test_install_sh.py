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


INSTALL_SCRIPT = Path(__file__).with_name("install.sh")
VERSION = "0.1.27"
TARGET = "aarch64-apple-darwin"
GITHUB_API = "https://api.github.com/repos/agtico/PfTerminal/releases"
GITHUB_DOWNLOAD = "https://github.com/agtico/PfTerminal/releases/download"


class InstallShTest(unittest.TestCase):
    def test_installer_has_no_stock_codex_release_or_home_defaults(self) -> None:
        source = INSTALL_SCRIPT.read_text(encoding="utf-8")

        self.assertNotIn("github.com/openai/codex", source)
        self.assertNotIn("api.github.com/repos/openai/codex", source)
        self.assertNotIn("releases.openai.com", source)
        self.assertNotIn('$HOME/.codex', source)
        self.assertIn('$HOME/.pfterminal', source)
        self.assertIn('BIN_PATH="$BIN_DIR/pfterminal"', source)

    def test_metadata_failure_names_pfterminal_and_stops(self) -> None:
        result, requests = run_installer(VERSION, metadata_failure=True)

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [f"{GITHUB_API}/tags/rust-v{VERSION}"],
        )
        self.assertIn(
            f"Could not fetch GitHub release metadata for PFTerminal {VERSION}",
            result.stderr,
        )

    def test_exact_release_uses_only_pfterminal_assets(self) -> None:
        result, requests = run_installer(VERSION)

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                f"{GITHUB_API}/tags/rust-v{VERSION}",
                f"{GITHUB_DOWNLOAD}/rust-v{VERSION}/pfterminal-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {VERSION}", result.stdout)

    def test_latest_release_reuses_github_metadata(self) -> None:
        result, requests = run_installer("latest")

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(
            requests,
            [
                f"{GITHUB_API}/latest",
                f"{GITHUB_DOWNLOAD}/rust-v{VERSION}/pfterminal-package_SHA256SUMS",
            ],
        )
        self.assertIn(f"Resolved version: {VERSION}", result.stdout)

    def test_verified_package_installs_pfterminal_launcher(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive, checksums, metadata = create_package_release(root)

            result, requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata,
                archive_path=archive,
                checksum_path=checksums,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                requests,
                [
                    f"{GITHUB_API}/tags/rust-v{VERSION}",
                    f"{GITHUB_DOWNLOAD}/rust-v{VERSION}/pfterminal-package_SHA256SUMS",
                    f"{GITHUB_DOWNLOAD}/rust-v{VERSION}/pfterminal-package-{TARGET}.tar.gz",
                ],
            )
            launcher = root / "install-bin" / "pfterminal"
            self.assertTrue(os.access(launcher, os.X_OK))
            launcher_source = launcher.read_text(encoding="utf-8")
            self.assertIn("$HOME/.pfterminal", launcher_source)
            self.assertNotIn("$HOME/.codex", launcher_source)
            completed = subprocess.run(
                [str(launcher), "--version"],
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(completed.stdout.strip(), f"pfterminal {VERSION}")

    def test_corrupt_package_is_rejected_before_install(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive, checksums, metadata = create_package_release(root)
            archive.write_bytes(b"corrupt")

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata,
                archive_path=archive,
                checksum_path=checksums,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("checksum did not match expected digest", result.stderr)
            self.assertFalse((root / "install-bin" / "pfterminal").exists())

    def test_wrong_binary_version_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive, checksums, metadata = create_package_release(
                root,
                binary_version="0.1.26",
            )

            result, _requests = run_installer_in(
                root,
                VERSION,
                metadata_json=metadata,
                archive_path=archive,
                checksum_path=checksums,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                f"did not report expected version {VERSION}",
                result.stderr,
            )
            self.assertFalse((root / "install-bin" / "pfterminal").exists())


def run_installer(
    release: str,
    *,
    metadata_failure: bool = False,
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    with tempfile.TemporaryDirectory() as temp_dir:
        return run_installer_in(
            Path(temp_dir),
            release,
            metadata_failure=metadata_failure,
        )


def run_installer_in(
    root: Path,
    release: str,
    *,
    metadata_failure: bool = False,
    metadata_json: str | None = None,
    archive_path: Path | None = None,
    checksum_path: Path | None = None,
) -> tuple[subprocess.CompletedProcess[str], list[str]]:
    bin_dir = root / "bin"
    bin_dir.mkdir(exist_ok=True)
    request_log = root / "requests.log"
    fake_curl = bin_dir / "curl"
    fake_curl.write_text(
        textwrap.dedent(
            """\
            #!/bin/sh
            url=""
            output=""
            previous=""
            for arg in "$@"; do
              case "$arg" in
                https://*) url="$arg" ;;
              esac
              if [ "$previous" = "-o" ]; then
                output="$arg"
              fi
              previous="$arg"
            done
            printf '%s\n' "$url" >>"$PFTERMINAL_TEST_REQUEST_LOG"

            case "$url" in
              https://api.github.com/repos/agtico/PfTerminal/releases/*)
                if [ "$PFTERMINAL_TEST_METADATA_FAILURE" = "1" ]; then
                  exit 22
                fi
                printf '%s\n' "$PFTERMINAL_TEST_METADATA_JSON"
                ;;
              https://github.com/agtico/PfTerminal/releases/download/*/pfterminal-package_SHA256SUMS)
                if [ -n "$PFTERMINAL_TEST_CHECKSUM_PATH" ]; then
                  cp "$PFTERMINAL_TEST_CHECKSUM_PATH" "$output"
                else
                  exit 22
                fi
                ;;
              https://github.com/agtico/PfTerminal/releases/download/*/pfterminal-package-*.tar.gz)
                if [ -n "$PFTERMINAL_TEST_ARCHIVE_PATH" ]; then
                  cp "$PFTERMINAL_TEST_ARCHIVE_PATH" "$output"
                else
                  exit 22
                fi
                ;;
              *)
                exit 22
                ;;
            esac
            """
        ),
        encoding="utf-8",
    )
    fake_curl.chmod(0o755)
    fake_uname = bin_dir / "uname"
    fake_uname.write_text(
        "#!/bin/sh\n"
        'case "$1" in\n'
        "  -s) printf 'Darwin\\n' ;;\n"
        "  -m) printf 'arm64\\n' ;;\n"
        "esac\n",
        encoding="utf-8",
    )
    fake_uname.chmod(0o755)

    home = root / "home"
    home.mkdir(exist_ok=True)
    env = os.environ.copy()
    env.update(
        {
            "PFTERMINAL_HOME": str(root / "pfterminal-home"),
            "PFTERMINAL_INSTALL_DIR": str(root / "install-bin"),
            "PFTERMINAL_NON_INTERACTIVE": "1",
            "PFTERMINAL_RELEASE": release,
            "PFTERMINAL_TEST_ARCHIVE_PATH": str(archive_path or ""),
            "PFTERMINAL_TEST_CHECKSUM_PATH": str(checksum_path or ""),
            "PFTERMINAL_TEST_METADATA_FAILURE": "1" if metadata_failure else "0",
            "PFTERMINAL_TEST_METADATA_JSON": metadata_json or release_metadata(),
            "PFTERMINAL_TEST_REQUEST_LOG": str(request_log),
            "HOME": str(home),
            "PATH": f"{bin_dir}:/usr/bin:/bin",
            "SHELL": "/bin/sh",
        }
    )
    result = subprocess.run(
        ["/bin/sh", str(INSTALL_SCRIPT)],
        capture_output=True,
        check=False,
        env=env,
        text=True,
    )
    requests = (
        request_log.read_text(encoding="utf-8").splitlines()
        if request_log.exists()
        else []
    )
    return result, requests


def create_package_release(
    root: Path,
    *,
    binary_version: str = VERSION,
) -> tuple[Path, Path, str]:
    package_dir = root / "package"
    (package_dir / "bin").mkdir(parents=True)
    (package_dir / "codex-path").mkdir()
    (package_dir / "codex-resources").mkdir()
    (package_dir / "codex-package.json").write_text("{}\n", encoding="utf-8")
    write_executable(
        package_dir / "bin" / "pfterminal",
        f"#!/bin/sh\nprintf 'pfterminal {binary_version}\\n'\n",
    )
    write_executable(
        package_dir / "bin" / "codex-code-mode-host",
        "#!/bin/sh\nexit 0\n",
    )
    write_executable(package_dir / "codex-path" / "rg", "#!/bin/sh\nexit 0\n")

    asset = f"pfterminal-package-{TARGET}.tar.gz"
    archive_path = root / asset
    with tarfile.open(archive_path, "w:gz") as archive:
        for path in package_dir.iterdir():
            archive.add(path, arcname=path.name)

    archive_digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    checksum_path = root / "pfterminal-package_SHA256SUMS"
    checksum_path.write_text(f"{archive_digest}  {asset}\n", encoding="utf-8")
    checksum_digest = hashlib.sha256(checksum_path.read_bytes()).hexdigest()
    metadata = json.dumps(
        {
            "assets": [
                {"name": asset, "digest": f"sha256:{archive_digest}"},
                {
                    "name": "pfterminal-package_SHA256SUMS",
                    "digest": f"sha256:{checksum_digest}",
                },
            ],
            "tag_name": f"rust-v{VERSION}",
        }
    )
    return archive_path, checksum_path, metadata


def release_metadata() -> str:
    return json.dumps(
        {
            "assets": [
                {
                    "name": f"pfterminal-package-{TARGET}.tar.gz",
                    "digest": f"sha256:{'a' * 64}",
                },
                {
                    "name": "pfterminal-package_SHA256SUMS",
                    "digest": f"sha256:{'b' * 64}",
                },
            ],
            "tag_name": f"rust-v{VERSION}",
        }
    )


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(0o755)


if __name__ == "__main__":
    unittest.main()

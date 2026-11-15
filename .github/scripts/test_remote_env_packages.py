"""Bounded Docker package recovery; no daemon, package install or real sleeps."""

import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "scripts/remote-env-packages.sh"

HARNESS = r"""
set -euo pipefail
source "$HELPER"
calls=0
docker() {
  [[ "$1" == exec && "$2" == "test container" ]] || return 99
  shift 2
  case "$*" in
    *APT::Update::Error-Mode=any*)
      [[ "$1" == timeout && "$2" == --kill-after=10s && "$3" == 180s ]] || return 99
      [[ "$*" == *'Acquire::Retries=1'* && "$*" == *'Acquire::http::Timeout=30'* ]] || return 99
      [[ "$*" == *'update &&'* && "$*" == *'install -y python3 zsh bubblewrap'* ]] || return 99
      calls=$((calls + 1))
      echo "APT $calls"
      if (( calls < SUCCESS_AT )); then
        echo "Could not resolve 'archive.ubuntu.com'" >&2
        return "$APT_STATUS"
      fi
      ;;
    *getent*)
      echo DIAGNOSTICS
      return 1
      ;;
    *sed*)
      [[ "$*" == *http://azure.archive.ubuntu.com/ubuntu* ]] || return 99
      [[ "$*" == */etc/apt/sources.list.d/ubuntu.sources* ]] || return 99
      echo MIRROR
      return "$MIRROR_STATUS"
      ;;
    *) echo "unexpected docker command" >&2; return 99 ;;
  esac
}
sleep() { echo "SLEEP $1"; }
# Use the same conditional invocation as setup_remote_env.
if install_remote_env_packages "test container"; then
  echo READY
else
  exit 1
fi
"""


class RemoteEnvPackagesTests(unittest.TestCase):
    def run_setup(self, success_at, apt_status=100, mirror_status=0):
        return subprocess.run(
            ["bash", "-c", HARNESS],
            env={
                **os.environ,
                "HELPER": str(HELPER),
                "SUCCESS_AT": str(success_at),
                "APT_STATUS": str(apt_status),
                "MIRROR_STATUS": str(mirror_status),
            },
            capture_output=True,
            text=True,
            timeout=5,
        )

    def test_success_does_not_retry_or_change_mirror(self):
        result = self.run_setup(1)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), ["APT 1", "READY"])

    def test_dns_failure_recovers_after_old_three_attempt_limit(self):
        result = self.run_setup(4)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout.splitlines(),
            [
                "APT 1",
                "MIRROR",
                "SLEEP 30",
                "APT 2",
                "SLEEP 60",
                "APT 3",
                "SLEEP 120",
                "APT 4",
                "READY",
            ],
        )
        self.assertIn("Could not resolve", result.stderr)
        self.assertIn("DIAGNOSTICS", result.stderr)

    def test_persistent_failure_never_reports_ready(self):
        result = self.run_setup(99)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout.count("APT "), 6)
        self.assertEqual(result.stdout.count("MIRROR"), 1)
        self.assertEqual(
            [line for line in result.stdout.splitlines() if line.startswith("SLEEP")],
            ["SLEEP 30", "SLEEP 60", "SLEEP 120", "SLEEP 120", "SLEEP 120"],
        )
        self.assertNotIn("READY", result.stdout)
        self.assertIn("after 6 attempts; remote tests cannot start", result.stderr)

    def test_timeout_is_retried_and_still_fails_when_exhausted(self):
        result = self.run_setup(99, apt_status=124)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stdout.count("APT "), 6)
        self.assertNotIn("READY", result.stdout)

    def test_mirror_configuration_failure_is_fatal(self):
        result = self.run_setup(2, mirror_status=2)
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("SLEEP", result.stdout)
        self.assertNotIn("READY", result.stdout)
        self.assertIn("Could not configure", result.stderr)


if __name__ == "__main__":
    unittest.main()

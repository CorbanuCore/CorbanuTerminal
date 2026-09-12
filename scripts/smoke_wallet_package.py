#!/usr/bin/env python3
"""Check automatic wallet startup using the production client and packaged daemon."""

import argparse
import os
from pathlib import Path
import queue
import shutil
import signal
import subprocess
import tempfile
import threading


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package-dir", type=Path, required=True)
    parser.add_argument("--probe", type=Path, required=True)
    args = parser.parse_args()
    suffix = ".exe" if os.name == "nt" else ""
    package_bin = args.package_dir.resolve() / "bin"
    daemon = package_bin / f"corbanu-walletd{suffix}"
    if not daemon.is_file():
        raise SystemExit(f"Packaged wallet daemon missing: {daemon}")
    if (package_bin / f"pfterminal-walletd{suffix}").exists():
        raise SystemExit("Legacy daemon would mask canonical package lookup failures")
    probe = package_bin / f"wallet-package-probe{suffix}"
    if probe.exists():
        raise SystemExit(f"Refusing to replace existing file: {probe}")
    shutil.copy2(args.probe, probe)
    try:
        with tempfile.TemporaryDirectory(prefix="cw-") as temp:
            proc = subprocess.Popen(
                [str(probe), str(Path(temp) / "h")],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                start_new_session=os.name != "nt",
            )
            output = queue.Queue()

            def collect():
                for line in proc.stdout:
                    output.put(line)
                output.put(None)

            threading.Thread(target=collect, daemon=True).start()
            try:
                line = output.get(timeout=30)
                if line is None or line.strip() != "wallet-client-package-ok":
                    raise RuntimeError(f"Packaged client startup failed: {line}")
                print(line.strip())
            finally:
                if os.name == "nt":
                    subprocess.run(
                        ["taskkill", "/PID", str(proc.pid), "/T", "/F"],
                        capture_output=True,
                        check=False,
                    )
                else:
                    try:
                        os.killpg(proc.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                proc.wait(timeout=10)
                proc.stdin.close()
                proc.stdout.close()
    finally:
        probe.unlink()


if __name__ == "__main__":
    main()

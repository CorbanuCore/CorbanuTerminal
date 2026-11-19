#!/usr/bin/env python3
"""Emit fixed numeric resource fields on stderr; missing fields are unavailable."""

import json
import math
import os
import re
import shutil
import sys
import time
from pathlib import Path

MEMINFO = Path("/proc/meminfo")
CGROUP = Path("/sys/fs/cgroup")
MEMORY_FIELDS = {
    "MemTotal": "memory_total_bytes",
    "MemAvailable": "memory_available_bytes",
    "SwapTotal": "swap_total_bytes",
    "SwapFree": "swap_free_bytes",
}
CGROUP_FIELDS = {
    "memory.current": "cgroup_memory_current_bytes",
    "memory.max": "cgroup_memory_limit_bytes",
    "memory.swap.current": "cgroup_swap_current_bytes",
    "memory.swap.max": "cgroup_swap_limit_bytes",
}
PREFIX = "Bazel resource sample: "


def read_counter_file(path: Path) -> str:
    try:
        with path.open(encoding="ascii") as source:
            return source.read(65536)
    except (OSError, UnicodeError):
        return ""


def collect_metrics() -> dict[str, int | float]:
    metrics: dict[str, int | float] = {}
    for line in read_counter_file(MEMINFO).splitlines():
        match = re.fullmatch(r"([A-Za-z]+):\s+([0-9]{1,20})\s+kB", line)
        if match and match[1] in MEMORY_FIELDS:
            metrics[MEMORY_FIELDS[match[1]]] = int(match[2]) * 1024
    for filename, key in CGROUP_FIELDS.items():
        value = read_counter_file(CGROUP / filename).strip()
        # The cgroup string "max" denotes an unbounded limit, not zero.
        if re.fullmatch(r"[0-9]{1,20}", value):
            metrics[key] = int(value)
    for line in read_counter_file(CGROUP / "memory.events").splitlines():
        match = re.fullmatch(r"(low|high|max|oom|oom_kill) ([0-9]{1,20})", line)
        if match:
            metrics[f"cgroup_memory_{match[1]}_events"] = int(match[2])
    for key, path in (
        ("workspace_disk_free_bytes", Path.cwd()),
        ("bazel_disk_free_bytes", os.environ.get("BAZEL_OUTPUT_USER_ROOT")),
    ):
        if path:
            try:
                metrics[key] = shutil.disk_usage(path).free
            except (OSError, ValueError):
                pass
    try:
        for minutes, value in zip((1, 5, 15), os.getloadavg(), strict=True):
            if math.isfinite(value) and value >= 0:
                metrics[f"load_{minutes}_minutes"] = value
    except OSError:
        pass
    return metrics


def emit_sample() -> bool:
    try:
        metrics = collect_metrics()
        message = PREFIX + json.dumps(metrics, sort_keys=True, allow_nan=False)
    except Exception:
        # Never include exception text: it can contain a path or environment value.
        message = "Bazel resource metrics unavailable"
    try:
        print(message, file=sys.stderr, flush=True)
        return True
    except OSError:
        return False


def main() -> None:
    if len(sys.argv) != 2 or not re.fullmatch(r"[1-9][0-9]{0,19}", sys.argv[1]):
        return
    parent_pid = int(sys.argv[1])
    # EXIT cleanup normally terminates and reaps us. Also bound our lifetime if
    # the wrapper is killed before its trap runs, including during startup.
    while os.getppid() == parent_pid:
        if not emit_sample():
            return
        time.sleep(15)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass

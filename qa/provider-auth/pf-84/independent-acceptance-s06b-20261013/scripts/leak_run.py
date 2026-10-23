#!/usr/bin/env python3
"""Fetch the four real keys from the installed vault helper into this process only (never printed, never argv),
then run leakscan.py in-process over the disposable homes, logs and this evidence folder.
usage: S=... EV=... [LEAK_ROOTS=a:b] python3 leak_run.py > captures/leak-scan-*.txt"""
import glob, os, runpy, subprocess, sys
KH = "/Users/Neo/.local/bin/corbanu.bak-20261010"
env = {k: v for k, v in os.environ.items() if k not in ("CORBANU_TEST_NO_NATIVE_KEYRING", "CORBANU_HOME", "CODEX_HOME", "PFTERMINAL_HOME")}
for name, label in (("REAL_ZAI_KEY", "provider/zai_api_key"), ("REAL_KIMI_KEY", "provider/kimi_api_key"),
                    ("REAL_OPENAI_KEY", "openai-api-key"), ("REAL_CLAUDE_PLAN_TOKEN", "claude-plan-test-token")):
    v = subprocess.run([KH, "vault", "auth-helper", label], env=env, capture_output=True, text=True).stdout.strip()
    if not v:
        sys.exit(f"could not read {label}")
    os.environ[name] = v
S = os.environ["S"]
roots = os.environ.get("LEAK_ROOTS")
sys.argv = ["leakscan.py"] + (roots.split(":") if roots else [f"{S}/homes", f"{S}/fakehome", f"{S}/logs", os.environ["EV"]] + sorted(glob.glob(f"{S}/fh-*")))
runpy.run_path(os.path.join(os.path.dirname(os.path.abspath(__file__)), "leakscan.py"), run_name="__main__")

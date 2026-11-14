"""Assisted live setup in a dedicated profile; never print/store credential values.

The profile is retained privately for restart testing. Authentication screen text
is printed only for the operator's browser handoff and is not put in QA artifacts.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

from functional_harness import Session, base_config


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("start", "openai-finish", "claude-token", "finish"))
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--token-file", type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    if args.phase == "start":
        args.root.mkdir(parents=True, exist_ok=False, mode=0o700)
    session = Session(args.candidate.resolve(), args.repo.resolve(), args.root, args.evidence,
                      {"RUST_LOG": "warn"})
    ledger_path = args.root / "completed-phases.json"
    phases = json.loads(ledger_path.read_text()) if ledger_path.exists() else []
    if args.phase == "start":
        (session.home / "config.toml").write_text('model="gpt-6-astra"\nmodel_provider="openai"\nmodel_reasoning_effort="high"\n' + base_config(args.repo.resolve()))
        # Fresh onboarding may precede the chat header; start directly then wait
        # for the provider entry instead of treating absence of chat as failure.
        import shlex
        session.tmux("new-session", "-d", "-s", "qa", "-x", "140", "-y", "44",
                     shlex.join([str(args.candidate.resolve()), "--no-alt-screen", "-C", str(args.repo.resolve())]))
        session.wait("OpenAI")
        session.choose("OpenAI")
        print(session.view(), flush=True)
        if "Set up with OpenAI account" in session.view():
            session.choose("Set up with OpenAI account")
        elif "Preparing device code login" not in session.view() and "Finish signing in via your browser" not in session.view():
            session.choose("OpenAI account")
        session.wait("https://auth.openai.com/codex/device")
        print(session.view(), flush=True)
    elif args.phase == "openai-finish":
        if "Signed in with your ChatGPT account" in session.view():
            session.key("Enter")
            session.wait("Choose a provider account")
            session.choose("Done")
        elif "Choose a provider account" in session.view():
            session.choose("Done")
        session.wait("/model to change")
        if "Select Model and Effort" not in session.view():
            session.picker("OpenAI")
        session.choose("gpt-6-astra")
        session.wait("Select Reasoning")
        session.choose("High")
        session.submit("Reply only with PF58, FRESH, OPENAI, OK joined by underscores. Do not use tools.")
        session.wait("PF58_FRESH_OPENAI_OK", timeout=120)
        session.save("fresh-openai-live")
        phases.append("openai-finish")
        session.receipt({"passed": True, "flow": "fresh OpenAI onboarding plus same-process live reply"})
        print("Fresh OpenAI setup and live reply passed.")
    elif args.phase == "claude-token":
        if not args.token_file:
            raise SystemExit("--token-file required; do not pass a token argument")
        # Read only the user-designated token file; never include it in argv,
        # logs, receipt, transcript, prompt text, or a new plaintext test file.
        token = args.token_file.read_text().strip()
        if not token or "\n" in token:
            raise SystemExit("Expected a single token in the designated file")
        session.manager()
        session.choose("Claude Account")
        session.choose("Set up with Claude account")
        session.wait("Claude Plan authentication")
        session.choose("Long-lived subscription token")
        session.wait("Long-lived token — masked")
        subprocess.run(["tmux", "-S", session.socket, "load-buffer", "-b", "pf58-live", "-"],
                       input=token, text=True, check=True, capture_output=True, env=session.env)
        session.tmux("paste-buffer", "-p", "-d", "-b", "pf58-live", "-t", "qa")
        session.wait("••")
        session.key("Enter")
        session.wait("Configure providers and control")
        session.choose("Claude Account", confirm=False)
        session.wait("Enabled · configured")
        assert token not in session.view(True), "secret appeared in rendered history"
        del token
        session.key("Escape")
        session.picker("Claude Plan")
        session.choose("Claude Fable 5.1 Plan")
        session.wait("Select Reasoning")
        session.choose("High")
        session.submit("Reply only with PF58, FRESH, CLAUDE, OK joined by underscores. Do not use tools.")
        session.wait("PF58_FRESH_CLAUDE_OK", timeout=120)
        session.save("fresh-claude-live")
        phases.append("claude-token")
        print("Fresh managed Claude token setup and live reply passed.")
    else:
        # Never turn a restart-only run into a fabricated fresh-login receipt.
        assert {"openai-finish", "claude-token"}.issubset(phases), "Complete both live setup phases before finish"
        session.exit()
        session.start()
        session.manager()
        session.save("live-restart-providers")
        session.key("Escape")
        session.submit("Reply only with PF58, RESTART, LIVE, OK joined by underscores. Do not use tools.")
        session.wait("PF58_RESTART_LIVE_OK", timeout=120)
        session.save("live-restart-reply")
        session.exit()
        session.close()
        session.receipt({"passed": True, "flow": "fresh live onboarding, separate Claude token setup, restart reply"})
        print("Live restart passed. Dedicated profile retained privately for acceptance.")
    ledger_path.write_text(json.dumps(phases))


if __name__ == "__main__":
    main()

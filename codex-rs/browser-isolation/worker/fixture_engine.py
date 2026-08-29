#!/usr/bin/python3
"""Test-only engine state machine; never copied into the application image."""
import json
import os
import sys
import time
from pathlib import Path

path = Path(os.environ["PF30_FIXTURE_STATE"])
state = json.loads(path.read_text())
args = sys.argv[1:]
state["calls"].append(args)
result = None
code = 0
if args[:2] == ["image", "inspect"]:
    result = [state["image"]]
elif args[0] == "build":
    result = state.get("build_id", state["image"]["Id"])
elif args[0] == "create":
    state["created"] = True
    label = args[args.index("--label") + 1].split("=", 1)[1]
    state["container"]["Config"]["Labels"]["org.corbanu.browser.owner"] = label
    result = "fixture-id"
elif args[:2] == ["container", "inspect"]:
    if state["created"]:
        result = [state["container"]]
    else:
        code = 1
elif args[0] in ["start", "restart"]:
    state["container"]["State"]["Running"] = True
elif args[0] == "exec":
    if state["failures"]:
        state["failures"] -= 1
        code = 1
    else:
        result = {"type": "healthy", "version": 1}
elif args[0] == "rm":
    state["created"] = False
else:
    code = 1
path.write_text(json.dumps(state))
if args[0] == "exec" and state.get("stall"):
    time.sleep(10)
if args[0] == "build" and result is not None:
    print(result)
elif result is not None:
    print(json.dumps(result))
sys.exit(code)

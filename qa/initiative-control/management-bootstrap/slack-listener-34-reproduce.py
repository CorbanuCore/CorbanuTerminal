"""Private-copy listener diagnostic; emits only fixed metadata, never error text."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from unittest.mock import patch

import decision_alerts as a
import decision_manager as m

LIVE = Path("/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/state/slack-operator")
FILES = ("transport.json", "alerts.json", "replies.json", "supervisor.json",
         ".slack.lock", ".transport.lock", ".ingress.fence", ".listener.owner.lock", ".listener.runtime.lock")


def fingerprints(root):
    return {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
            for name in FILES if (root / name).is_file()}


def child():
    # Only the exception propagated into listener_child is retained. No args,
    # exception message, locals, SDK payload, credential or source text.
    output = Path(sys.argv[5])
    def trace(frame, event, arg):
        if event == "exception" and frame.f_code.co_name == "listener_child":
            kind, error, tb = arg
            locations = []
            while tb:
                locations.append([Path(tb.tb_frame.f_code.co_filename).name,
                                  tb.tb_frame.f_code.co_name, tb.tb_lineno])
                tb = tb.tb_next
            output.write_text(json.dumps(dict(exception=kind.__name__, locations=locations,
                                              missing_slack_sdk=isinstance(error, ModuleNotFoundError)
                                              and error.name == "slack_sdk")))
            os.chmod(output, 0o600)
        return trace
    def deny_network(event, args):
        if event == "socket.connect":
            raise PermissionError()
    sys.addaudithook(deny_network)
    sys.settrace(trace)
    m.listener_child(sys.argv[2], int(sys.argv[3]), int(sys.argv[4]))


def main():
    destination = Path(sys.argv[1])
    destination.mkdir(mode=0o700)
    before = fingerprints(LIVE)
    for name in before:
        shutil.copyfile(LIVE / name, destination / name)
        os.chmod(destination / name, 0o600)
    store = a.Store(destination)
    journal = store.read("transport")
    print(json.dumps(dict(source="live-read-only", ingress=journal["ingress"],
                          fence=(LIVE / ".ingress.fence").stat().st_size,
                          watermark=journal["watermark"], epoch=journal["lifecycle"]["epoch"],
                          listener_exits=m.project_status(a.Store(LIVE), m.utc_now(), True)["listener_exits"])))
    # Copied inodes differ; relocate only the two birth pins in the COPY.
    for name, pin in ((".listener.runtime.lock", journal["runtime_guard"]),
                      (".listener.owner.lock", journal["lifecycle"])):
        info = (destination / name).stat()
        pin["file" if name == ".listener.runtime.lock" else "owner_file"] = [info.st_dev, info.st_ino]
    store.write("transport", journal)
    failure = destination.parent / (destination.name + "-failure.json")
    popen = subprocess.Popen
    def launch(command, **kwargs):
        command = [command[0], "-B", str(Path(__file__).resolve()), "child", *command[4:], str(failure)]
        return popen(command, **kwargs)
    incoming, writer = os.pipe()
    try:
        with os.fdopen(incoming, "rb") as source, open(os.devnull, "wb") as sink:
            os.write(writer, json.dumps(dict(binding=journal["binding"])).encode() + b"\n")
            with patch.object(m.subprocess, "Popen", side_effect=launch):
                m.main(["listen", "--store", str(destination), "--live"], stdin=source, stdout=sink)
    except Exception as error:
        print(json.dumps(dict(source="copy-parent", exception=type(error).__name__)))
    finally:
        os.close(writer)
    if failure.exists():
        print(failure.read_text())
    after = store.read("transport")
    protected = ("ingress", "watermark", "binding", "gap_reviews")
    print(json.dumps(dict(source="copy-after", listener_exits=m.project_status(store, m.utc_now(), True)["listener_exits"],
                          protected_unchanged=all(after[key] == journal[key] for key in protected),
                          fence_unchanged=hashlib.sha256((destination / ".ingress.fence").read_bytes()).hexdigest() == before[".ingress.fence"],
                          live_unchanged=fingerprints(LIVE) == before)))
    (destination.parent / "live-before.json").write_text(json.dumps(before))


if __name__ == "__main__":
    child() if sys.argv[1] == "child" else main()

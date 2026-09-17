"""Disposable before/after evidence; no live store, profile or Slack connection."""
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import types
from unittest.mock import patch

BASE = "55dd447f41fb1c00b73d74d03bcbed8a22dd580c"
ROOT = Path(__file__).resolve().parents[3]


def child(version, scenario, control):
    if version == "base":
        for name in ("slack_transport", "decision_manager"):
            path = "scripts/initiative_control/" + name + ".py"
            source = subprocess.check_output(["git", "show", BASE + ":" + path], cwd=ROOT)
            module = types.ModuleType(name)
            module.__file__ = str(ROOT / path)
            sys.modules[name] = module
            exec(compile(source, module.__file__, "exec"), module.__dict__)
    import slack_transport as s
    import decision_manager as m
    import test_slack_transport as fixtures

    case = fixtures.LiveFixture("runTest")
    case.setUp()
    try:
        if scenario == "poison":
            case.sending()
            case.callback(fixtures.payload("EvUnknownDelete", subtype="message_deleted", deleted_ts="109.000001"))
            case.callback(fixtures.payload("EvFollowingHuman"))
            journal = case.store.read("transport")
            print(json.dumps(dict(version=version, scenario=scenario, active=case.transport.active,
                fence=s.ingress_count(case.store), ingress=journal["ingress"], acks=len(case.acks),
                human_events=len(journal["events"]), quarantine=journal.get("quarantine"))), flush=True)
            return
        case.owner.close()
        callbacks = [0]
        callback = case.transport.callback
        def observed_callback(*args):
            callbacks[0] += 1
            return callback(*args)
        locked = threading.Event()
        def hold_transport():
            with s.locked(case.store):
                locked.set()
                threading.Event().wait()  # The dedicated child exit reaps this fixture thread.
        def connect():
            threading.Thread(target=hold_transport, daemon=True).start()
            assert locked.wait(5)

        guard = s.runtime_file(case.store, fixtures.PIN)
        # Observe only the disposable fixture after listener_child calls os._exit.
        exit_process = os._exit
        def observed_exit(code):
            journal = case.store.read("transport")
            print(json.dumps(dict(version=version, scenario=scenario, child_exit=code,
                fence=s.ingress_count(case.store), ingress=journal["ingress"],
                sdk_callbacks=callbacks[0], events=len(journal["events"]))), flush=True)
            exit_process(code)
        with (patch.object(s, "Transport", return_value=case.transport),
              patch.object(case.transport, "callback", side_effect=observed_callback),
              patch.object(fixtures.SocketModeClient, "connect", side_effect=connect),
              patch.object(fixtures.SocketModeClient, "is_connected", return_value=True),
              patch.object(os, "_exit", side_effect=observed_exit)):
            m.listener_child(str(case.root), guard, int(control))
    finally:
        case.tearDown()
        case.doCleanups()


def main():
    if len(sys.argv) > 1:
        child(*sys.argv[1:])
        return
    from test_decision_alerts import PIN
    for version in ("base", "revised"):
        for scenario in ("poison", "three"):
            reader, writer = os.pipe()
            try:
                process = subprocess.Popen([sys.executable, "-B", __file__, version, scenario, str(reader)],
                    pass_fds=(reader,), stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                output, errors = process.communicate(json.dumps(dict(
                    binding=PIN, seconds=60, ongoing=True)).encode() + b"\n", timeout=30)
                expected = 1 if scenario == "three" else 0
                if process.returncode != expected or errors:
                    raise AssertionError((version, scenario, process.returncode, errors.decode()))
                rows = [json.loads(line) for line in output.splitlines()]
                print(json.dumps(dict(version=version, scenario=scenario,
                                      returncode=process.returncode, records=rows)), flush=True)
                result = rows[-1]
                if scenario == "three":
                    assert result["fence"] == 3 and result["ingress"] == result["sdk_callbacks"] == result["events"] == 0
                    if version == "revised":
                        assert rows[-2]["failure"] == dict(reason="transport-busy", stage="session-renew",
                            callbacks=0, disconnect_marks=3, quarantined=0)
                elif version == "base":
                    assert result["active"] is False and result["fence"] == 2 and result["ingress"] == result["acks"] == result["human_events"] == 0
                else:
                    assert result["active"] is True and result["fence"] == result["ingress"] == result["acks"] == 2
                    assert result["human_events"] == result["quarantine"]["total"] == 1
            finally:
                os.close(reader)
                os.close(writer)


if __name__ == "__main__":
    main()

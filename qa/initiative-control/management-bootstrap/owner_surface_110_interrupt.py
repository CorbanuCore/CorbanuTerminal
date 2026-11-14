"""Demonstrate post-hold interruption against a disposable synthetic transport."""
import json
from unittest.mock import patch

import decision_alerts as alerts
from test_slack_transport import TransportTests, ui_evidence


def main():
    case = TransportTests("test_qualification_restores_prior_hold_after_interrupt_auth_and_final_write_failure")
    case.setUp()
    try:
        case.sending()
        before = case.store.read("transport")
        during = []
        def interrupted():
            during.append(case.store.read("transport")["hold"])
            raise KeyboardInterrupt()
        with patch.object(case.transport.web(), "auth_test", side_effect=interrupted):
            try:
                case.transport.qualify(ui_evidence())
            except KeyboardInterrupt:
                pass
            else:
                raise AssertionError("interruption not propagated")
        after = alerts.Store(case.root).read("transport")
        assert during == ["qualifying"] and before == after
        case.transport.qualify(ui_evidence())
        assert case.store.read("transport")["hold"] is None
        print(json.dumps(dict(before_hold=before["hold"], during_hold=during[0],
                              after_hold=after["hold"], reopened_journal_unchanged=before == after,
                              interrupted_by="KeyboardInterrupt", retry="qualified",
                              stores="disposable", credentials="synthetic"), sort_keys=True))
    finally:
        case.doCleanups()


if __name__ == "__main__":
    main()

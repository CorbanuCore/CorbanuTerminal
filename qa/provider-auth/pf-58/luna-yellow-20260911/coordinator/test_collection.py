import unittest
from collect_results import run_inventory, execution_disposition, redact

class CollectionTests(unittest.TestCase):
    def test_replay_retains_original(self):
        runs = dict(run_inventory([{'case': 5, 'run_name': 'case-05-replay'}]))
        self.assertEqual(runs['case-05'], 5)
        self.assertEqual(runs['case-05-replay'], 5)

    def test_timeout_is_not_pending_or_pass(self):
        for status in ('pending', 'passed', 'blocked', 'incomplete'):
            self.assertEqual(execution_disposition({'status': status}, {'exit_code': 'timeout'}), 'timeout')

    def test_failed_process_cannot_pass(self):
        self.assertEqual(execution_disposition({'status': 'passed'}, {'exit_code': 1}), 'blocked')

    def test_complete_result_preserved(self):
        for status in ('passed', 'failed', 'blocked'):
            self.assertEqual(execution_disposition({'status': status}, {'exit_code': 0}), status)

    def test_redaction(self):
        self.assertNotIn('someone@example.com', redact('someone@example.com'))
        self.assertNotIn('ABCD-12345', redact('ABCD-12345'))
        self.assertNotIn('sk-abcdefghijklmnop', redact('sk-abcdefghijklmnop'))

if __name__ == '__main__':
    unittest.main()

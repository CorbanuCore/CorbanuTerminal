from __future__ import annotations

import unittest

from pipeflow import PipelineRunner


class VisibleIntegrationTests(unittest.TestCase):
    def test_env_disabled_unavailable_source_does_not_break_pipeline(self) -> None:
        config = {
            "tasks": {
                "fetch": {"uses": "explode", "enabled": True, "params": {"message": "upstream unavailable"}},
                "final": {"uses": "identity", "params": {"value": "ready"}},
            }
        }
        env = {"PIPEFLOW__TASKS__FETCH__ENABLED": "false"}

        result = PipelineRunner(config, env=env).run()

        self.assertEqual(result["outputs"], {"final": "ready"})
        self.assertEqual(result["order"], ["final"])


if __name__ == "__main__":
    unittest.main()

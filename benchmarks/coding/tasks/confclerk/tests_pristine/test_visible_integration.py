from __future__ import annotations

import unittest

from confclerk.loader import load_config
from confclerk.template import TemplateRenderer


class VisibleIntegrationTests(unittest.TestCase):
    def test_env_disabled_preview_does_not_render_missing_partial_value(self) -> None:
        config = load_config(env={"CONFCLERK__FEATURES__PREVIEW": "false"})
        template = "{{ service.name }}"
        if config.get("features", {}).get("preview"):
            template = "{{ preview.message }}"
        rendered = TemplateRenderer().render(template, {"service": {"name": "stable"}, **config})
        self.assertEqual(rendered, "stable")


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import unittest

from textwright import render, render_many
from textwright.ast import Node, text
from textwright.pipeline import pipeline_stage_1
from textwright.renderer import HtmlRenderer


class PipelineFeatureTests(unittest.TestCase):
    def test_render_many(self) -> None:
        self.assertEqual(render_many(["a", "b"]), ["<p>a</p>", "<p>b</p>"])

    def test_renderer_unknown_node(self) -> None:
        with self.assertRaises(Exception):
            HtmlRenderer().render(Node("unknown"))

    def test_node_walk(self) -> None:
        root = Node("root", children=[text("x")])
        self.assertEqual([node.type for node in root.walk()], ["root", "text"])

    def test_generated_pipeline_stage(self) -> None:
        self.assertIn("pipeline_stage_1_score", pipeline_stage_1({"score": 1}))

    def test_multiple_blocks(self) -> None:
        self.assertEqual(render("# A\nhello"), '<h1 id="a">A</h1><p>hello</p>')

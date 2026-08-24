from __future__ import annotations

import unittest

from textwright import render
from textwright.parser import parse
from textwright.renderer import render_html
from textwright.transform import slugify, transform


class TransformRendererTests(unittest.TestCase):
    def test_slugify(self) -> None:
        self.assertEqual(slugify("Hello, World!"), "hello-world")

    def test_transform_heading_id(self) -> None:
        root = transform(parse("# Hello"))
        self.assertEqual(root.children[0].attrs["id"], "hello")

    def test_table_columns(self) -> None:
        root = transform(parse("a|b|c"))
        self.assertEqual(root.children[0].attrs["columns"], 3)

    def test_render_paragraph_escapes_text(self) -> None:
        self.assertEqual(render("<x>"), "<p>&lt;x&gt;</p>")

    def test_render_emphasis(self) -> None:
        self.assertEqual(render("*hi*"), "<p><em>hi</em></p>")

    def test_render_link(self) -> None:
        self.assertEqual(render("[x](/a)"), '<p><a href="/a">x</a></p>')

    def test_render_list(self) -> None:
        self.assertEqual(render("- a"), "<ul><li>a</li></ul>")

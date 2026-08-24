from __future__ import annotations

import unittest

from textwright.lexer import lex
from textwright.parser import parse
from textwright.tokens import TokenKind


class LexerParserTests(unittest.TestCase):
    def test_lex_text(self) -> None:
        self.assertEqual(lex("abc")[0].value, "abc")

    def test_lex_escape(self) -> None:
        tokens = lex("a\\|b")
        self.assertNotIn(TokenKind.PIPE, [token.kind for token in tokens])

    def test_parse_paragraph(self) -> None:
        self.assertEqual(parse("hello").children[0].type, "paragraph")

    def test_parse_heading(self) -> None:
        node = parse("# hello").children[0]
        self.assertEqual(node.attrs["level"], 1)

    def test_parse_list(self) -> None:
        node = parse("- a\n- b")
        self.assertEqual(len(node.children[0].children), 2)

    def test_parse_table(self) -> None:
        node = parse("a|b")
        self.assertEqual(node.children[0].type, "table")

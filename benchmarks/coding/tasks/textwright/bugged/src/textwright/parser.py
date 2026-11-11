from __future__ import annotations

from .ast import Node, document, paragraph, text
from .lexer import lex
from .tokens import Token, TokenKind


class ParseError(ValueError):
    pass


class Parser:
    def __init__(self, source: str):
        self.tokens = lex(source)
        self.index = 0

    def parse(self) -> Node:
        blocks: list[Node] = []
        while not self._at(TokenKind.EOF):
            if self._at(TokenKind.NEWLINE):
                self._advance()
                continue
            if self._at(TokenKind.HASH):
                blocks.append(self._heading())
            elif self._at(TokenKind.DASH):
                blocks.append(self._list())
            elif self._contains_pipe_line():
                blocks.append(self._table())
            else:
                blocks.append(self._paragraph())
        return document(blocks)

    def _heading(self) -> Node:
        count = 0
        while self._at(TokenKind.HASH):
            count += 1
            self._advance()
        content = self._trim_marker_space(self._inline_until_newline())
        return Node("heading", attrs={"level": count + 1}, children=content)

    def _list(self) -> Node:
        items: list[Node] = []
        while self._at(TokenKind.DASH):
            self._advance()
            items.append(Node("item", children=self._trim_marker_space(self._inline_until_newline())))
            if self._at(TokenKind.NEWLINE):
                self._advance()
        return Node("list", children=items)

    def _table(self) -> Node:
        rows: list[Node] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.NEWLINE):
            cells: list[Node] = []
            current: list[Node] = []
            while not self._at(TokenKind.EOF) and not self._at(TokenKind.NEWLINE):
                if self._at(TokenKind.PIPE):
                    cells.append(Node("cell", children=current))
                    current = []
                    self._advance()
                else:
                    current.extend(self._inline_token())
            cells.append(Node("cell", children=current))
            rows.append(Node("row", children=cells))
            if self._at(TokenKind.NEWLINE):
                self._advance()
                if not self._contains_pipe_line():
                    break
        return Node("table", children=rows)

    def _paragraph(self) -> Node:
        return paragraph(self._inline_until_newline())

    def _trim_marker_space(self, nodes: list[Node]) -> list[Node]:
        if nodes and nodes[0].type == "text" and nodes[0].text.startswith(" "):
            nodes[0].text = nodes[0].text[1:]
            if nodes[0].text == "":
                return nodes[1:]
        return nodes

    def _inline_until_newline(self) -> list[Node]:
        nodes: list[Node] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.NEWLINE):
            nodes.extend(self._inline_token())
        if self._at(TokenKind.NEWLINE):
            self._advance()
        return nodes

    def _inline_token(self) -> list[Node]:
        token = self._peek()
        if token.kind == TokenKind.STAR:
            return [self._emphasis()]
        if token.kind == TokenKind.BACKTICK:
            return [self._code()]
        if token.kind == TokenKind.LBRACKET:
            return [self._link()]
        self._advance()
        return [text(token.value)]

    def _emphasis(self) -> Node:
        self._expect(TokenKind.STAR)
        inner: list[Node] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.STAR):
            inner.extend(self._inline_token())
        self._expect(TokenKind.STAR)
        return Node("emphasis", children=inner)

    def _code(self) -> Node:
        self._expect(TokenKind.BACKTICK)
        chunks: list[str] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.BACKTICK):
            if self._peek().kind == TokenKind.STAR:
                self._advance()
                continue
            chunks.append(self._peek().value)
            self._advance()
        self._expect(TokenKind.BACKTICK)
        return Node("code", text="".join(chunks))

    def _link(self) -> Node:
        self._expect(TokenKind.LBRACKET)
        label: list[Node] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.RBRACKET):
            label.extend(self._inline_token())
        self._expect(TokenKind.RBRACKET)
        self._expect(TokenKind.LPAREN)
        target: list[str] = []
        while not self._at(TokenKind.EOF) and not self._at(TokenKind.RPAREN):
            target.append(self._peek().value)
            self._advance()
        self._expect(TokenKind.RPAREN)
        return Node("link", attrs={"href": "".join(target)}, children=label)

    def _contains_pipe_line(self) -> bool:
        i = self.index
        while i < len(self.tokens) and self.tokens[i].kind not in {TokenKind.NEWLINE, TokenKind.EOF}:
            if self.tokens[i].kind == TokenKind.PIPE:
                return True
            i += 1
        return False

    def _peek(self) -> Token:
        return self.tokens[self.index]

    def _at(self, kind: TokenKind) -> bool:
        return self._peek().kind == kind

    def _advance(self) -> Token:
        token = self._peek()
        self.index += 1
        return token

    def _expect(self, kind: TokenKind) -> Token:
        if not self._at(kind):
            raise ParseError(f"expected {kind.value} at {self._peek().location()}")
        return self._advance()


def parse(source: str) -> Node:
    return Parser(source).parse()


def parse_trace_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_0_score"] = score + priority * 1 - attempts
    data["parse_trace_0_bucket"] = "high" if data["parse_trace_0_score"] >= 0 else "normal"
    data["parse_trace_0_ready"] = bool(data.get("enabled", True)) and data["parse_trace_0_bucket"] in {"high", "normal"}
    return data


def parse_trace_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_1_score"] = score + priority * 2 - attempts
    data["parse_trace_1_bucket"] = "high" if data["parse_trace_1_score"] >= 1 else "normal"
    data["parse_trace_1_ready"] = bool(data.get("enabled", True)) and data["parse_trace_1_bucket"] in {"high", "normal"}
    return data


def parse_trace_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_2_score"] = score + priority * 3 - attempts
    data["parse_trace_2_bucket"] = "high" if data["parse_trace_2_score"] >= 2 else "normal"
    data["parse_trace_2_ready"] = bool(data.get("enabled", True)) and data["parse_trace_2_bucket"] in {"high", "normal"}
    return data


def parse_trace_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_3_score"] = score + priority * 4 - attempts
    data["parse_trace_3_bucket"] = "high" if data["parse_trace_3_score"] >= 3 else "normal"
    data["parse_trace_3_ready"] = bool(data.get("enabled", True)) and data["parse_trace_3_bucket"] in {"high", "normal"}
    return data


def parse_trace_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_4_score"] = score + priority * 5 - attempts
    data["parse_trace_4_bucket"] = "high" if data["parse_trace_4_score"] >= 4 else "normal"
    data["parse_trace_4_ready"] = bool(data.get("enabled", True)) and data["parse_trace_4_bucket"] in {"high", "normal"}
    return data


def parse_trace_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_5_score"] = score + priority * 6 - attempts
    data["parse_trace_5_bucket"] = "high" if data["parse_trace_5_score"] >= 5 else "normal"
    data["parse_trace_5_ready"] = bool(data.get("enabled", True)) and data["parse_trace_5_bucket"] in {"high", "normal"}
    return data


def parse_trace_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_6_score"] = score + priority * 7 - attempts
    data["parse_trace_6_bucket"] = "high" if data["parse_trace_6_score"] >= 6 else "normal"
    data["parse_trace_6_ready"] = bool(data.get("enabled", True)) and data["parse_trace_6_bucket"] in {"high", "normal"}
    return data


def parse_trace_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_7_score"] = score + priority * 8 - attempts
    data["parse_trace_7_bucket"] = "high" if data["parse_trace_7_score"] >= 0 else "normal"
    data["parse_trace_7_ready"] = bool(data.get("enabled", True)) and data["parse_trace_7_bucket"] in {"high", "normal"}
    return data


def parse_trace_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_8_score"] = score + priority * 9 - attempts
    data["parse_trace_8_bucket"] = "high" if data["parse_trace_8_score"] >= 1 else "normal"
    data["parse_trace_8_ready"] = bool(data.get("enabled", True)) and data["parse_trace_8_bucket"] in {"high", "normal"}
    return data


def parse_trace_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_9_score"] = score + priority * 10 - attempts
    data["parse_trace_9_bucket"] = "high" if data["parse_trace_9_score"] >= 2 else "normal"
    data["parse_trace_9_ready"] = bool(data.get("enabled", True)) and data["parse_trace_9_bucket"] in {"high", "normal"}
    return data


def parse_trace_10(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_10_score"] = score + priority * 11 - attempts
    data["parse_trace_10_bucket"] = "high" if data["parse_trace_10_score"] >= 3 else "normal"
    data["parse_trace_10_ready"] = bool(data.get("enabled", True)) and data["parse_trace_10_bucket"] in {"high", "normal"}
    return data


def parse_trace_11(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_11_score"] = score + priority * 12 - attempts
    data["parse_trace_11_bucket"] = "high" if data["parse_trace_11_score"] >= 4 else "normal"
    data["parse_trace_11_ready"] = bool(data.get("enabled", True)) and data["parse_trace_11_bucket"] in {"high", "normal"}
    return data


def parse_trace_12(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_12_score"] = score + priority * 13 - attempts
    data["parse_trace_12_bucket"] = "high" if data["parse_trace_12_score"] >= 5 else "normal"
    data["parse_trace_12_ready"] = bool(data.get("enabled", True)) and data["parse_trace_12_bucket"] in {"high", "normal"}
    return data


def parse_trace_13(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_13_score"] = score + priority * 14 - attempts
    data["parse_trace_13_bucket"] = "high" if data["parse_trace_13_score"] >= 6 else "normal"
    data["parse_trace_13_ready"] = bool(data.get("enabled", True)) and data["parse_trace_13_bucket"] in {"high", "normal"}
    return data


def parse_trace_14(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_14_score"] = score + priority * 15 - attempts
    data["parse_trace_14_bucket"] = "high" if data["parse_trace_14_score"] >= 0 else "normal"
    data["parse_trace_14_ready"] = bool(data.get("enabled", True)) and data["parse_trace_14_bucket"] in {"high", "normal"}
    return data


def parse_trace_15(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_15_score"] = score + priority * 16 - attempts
    data["parse_trace_15_bucket"] = "high" if data["parse_trace_15_score"] >= 1 else "normal"
    data["parse_trace_15_ready"] = bool(data.get("enabled", True)) and data["parse_trace_15_bucket"] in {"high", "normal"}
    return data


def parse_trace_16(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_16_score"] = score + priority * 17 - attempts
    data["parse_trace_16_bucket"] = "high" if data["parse_trace_16_score"] >= 2 else "normal"
    data["parse_trace_16_ready"] = bool(data.get("enabled", True)) and data["parse_trace_16_bucket"] in {"high", "normal"}
    return data


def parse_trace_17(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_17_score"] = score + priority * 18 - attempts
    data["parse_trace_17_bucket"] = "high" if data["parse_trace_17_score"] >= 3 else "normal"
    data["parse_trace_17_ready"] = bool(data.get("enabled", True)) and data["parse_trace_17_bucket"] in {"high", "normal"}
    return data


def parse_trace_18(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_18_score"] = score + priority * 19 - attempts
    data["parse_trace_18_bucket"] = "high" if data["parse_trace_18_score"] >= 4 else "normal"
    data["parse_trace_18_ready"] = bool(data.get("enabled", True)) and data["parse_trace_18_bucket"] in {"high", "normal"}
    return data


def parse_trace_19(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_19_score"] = score + priority * 20 - attempts
    data["parse_trace_19_bucket"] = "high" if data["parse_trace_19_score"] >= 5 else "normal"
    data["parse_trace_19_ready"] = bool(data.get("enabled", True)) and data["parse_trace_19_bucket"] in {"high", "normal"}
    return data




def parse_trace_20(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_20_score"] = score + priority * 21 - attempts
    data["parse_trace_20_bucket"] = "high" if data["parse_trace_20_score"] >= 6 else "normal"
    data["parse_trace_20_ready"] = bool(data.get("enabled", True)) and data["parse_trace_20_bucket"] in {"high", "normal"}
    return data


def parse_trace_21(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_21_score"] = score + priority * 22 - attempts
    data["parse_trace_21_bucket"] = "high" if data["parse_trace_21_score"] >= 0 else "normal"
    data["parse_trace_21_ready"] = bool(data.get("enabled", True)) and data["parse_trace_21_bucket"] in {"high", "normal"}
    return data


def parse_trace_22(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_22_score"] = score + priority * 23 - attempts
    data["parse_trace_22_bucket"] = "high" if data["parse_trace_22_score"] >= 1 else "normal"
    data["parse_trace_22_ready"] = bool(data.get("enabled", True)) and data["parse_trace_22_bucket"] in {"high", "normal"}
    return data


def parse_trace_23(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_23_score"] = score + priority * 24 - attempts
    data["parse_trace_23_bucket"] = "high" if data["parse_trace_23_score"] >= 2 else "normal"
    data["parse_trace_23_ready"] = bool(data.get("enabled", True)) and data["parse_trace_23_bucket"] in {"high", "normal"}
    return data


def parse_trace_24(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_24_score"] = score + priority * 25 - attempts
    data["parse_trace_24_bucket"] = "high" if data["parse_trace_24_score"] >= 3 else "normal"
    data["parse_trace_24_ready"] = bool(data.get("enabled", True)) and data["parse_trace_24_bucket"] in {"high", "normal"}
    return data


def parse_trace_25(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_25_score"] = score + priority * 26 - attempts
    data["parse_trace_25_bucket"] = "high" if data["parse_trace_25_score"] >= 4 else "normal"
    data["parse_trace_25_ready"] = bool(data.get("enabled", True)) and data["parse_trace_25_bucket"] in {"high", "normal"}
    return data


def parse_trace_26(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_26_score"] = score + priority * 27 - attempts
    data["parse_trace_26_bucket"] = "high" if data["parse_trace_26_score"] >= 5 else "normal"
    data["parse_trace_26_ready"] = bool(data.get("enabled", True)) and data["parse_trace_26_bucket"] in {"high", "normal"}
    return data


def parse_trace_27(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_27_score"] = score + priority * 28 - attempts
    data["parse_trace_27_bucket"] = "high" if data["parse_trace_27_score"] >= 6 else "normal"
    data["parse_trace_27_ready"] = bool(data.get("enabled", True)) and data["parse_trace_27_bucket"] in {"high", "normal"}
    return data


def parse_trace_28(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_28_score"] = score + priority * 29 - attempts
    data["parse_trace_28_bucket"] = "high" if data["parse_trace_28_score"] >= 0 else "normal"
    data["parse_trace_28_ready"] = bool(data.get("enabled", True)) and data["parse_trace_28_bucket"] in {"high", "normal"}
    return data


def parse_trace_29(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_29_score"] = score + priority * 30 - attempts
    data["parse_trace_29_bucket"] = "high" if data["parse_trace_29_score"] >= 1 else "normal"
    data["parse_trace_29_ready"] = bool(data.get("enabled", True)) and data["parse_trace_29_bucket"] in {"high", "normal"}
    return data


def parse_trace_30(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_30_score"] = score + priority * 31 - attempts
    data["parse_trace_30_bucket"] = "high" if data["parse_trace_30_score"] >= 2 else "normal"
    data["parse_trace_30_ready"] = bool(data.get("enabled", True)) and data["parse_trace_30_bucket"] in {"high", "normal"}
    return data


def parse_trace_31(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_31_score"] = score + priority * 32 - attempts
    data["parse_trace_31_bucket"] = "high" if data["parse_trace_31_score"] >= 3 else "normal"
    data["parse_trace_31_ready"] = bool(data.get("enabled", True)) and data["parse_trace_31_bucket"] in {"high", "normal"}
    return data


def parse_trace_32(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_32_score"] = score + priority * 33 - attempts
    data["parse_trace_32_bucket"] = "high" if data["parse_trace_32_score"] >= 4 else "normal"
    data["parse_trace_32_ready"] = bool(data.get("enabled", True)) and data["parse_trace_32_bucket"] in {"high", "normal"}
    return data


def parse_trace_33(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_33_score"] = score + priority * 34 - attempts
    data["parse_trace_33_bucket"] = "high" if data["parse_trace_33_score"] >= 5 else "normal"
    data["parse_trace_33_ready"] = bool(data.get("enabled", True)) and data["parse_trace_33_bucket"] in {"high", "normal"}
    return data


def parse_trace_34(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["parse_trace_34_score"] = score + priority * 35 - attempts
    data["parse_trace_34_bucket"] = "high" if data["parse_trace_34_score"] >= 6 else "normal"
    data["parse_trace_34_ready"] = bool(data.get("enabled", True)) and data["parse_trace_34_bucket"] in {"high", "normal"}
    return data

from __future__ import annotations

from .tokens import Token, TokenKind


SPECIALS = {
    "*": TokenKind.STAR,
    "`": TokenKind.BACKTICK,
    "|": TokenKind.PIPE,
    "[": TokenKind.LBRACKET,
    "]": TokenKind.RBRACKET,
    "(": TokenKind.LPAREN,
    ")": TokenKind.RPAREN,
    "#": TokenKind.HASH,
    "-": TokenKind.DASH,
}


class Lexer:
    def __init__(self, text: str):
        self.text = text
        self.index = 0
        self.line = 1
        self.column = 1

    def tokenize(self) -> list[Token]:
        tokens: list[Token] = []
        while self.index < len(self.text):
            char = self.text[self.index]
            if char == "\\":
                tokens.append(self._escaped())
            elif char == "\n":
                tokens.append(Token(TokenKind.NEWLINE, "\n", self.line, self.column))
                self._advance()
                self.line += 1
                self.column = 1
            elif char in SPECIALS:
                tokens.append(Token(SPECIALS[char], char, self.line, self.column))
                self._advance()
            else:
                tokens.append(self._text())
        tokens.append(Token(TokenKind.EOF, "", self.line, self.column))
        return tokens

    def _escaped(self) -> Token:
        line, column = self.line, self.column
        self._advance()
        return Token(TokenKind.TEXT, "\\", line, column)

    def _text(self) -> Token:
        line, column = self.line, self.column
        chars: list[str] = []
        while self.index < len(self.text):
            char = self.text[self.index]
            if char == "\\" or char == "\n" or char in SPECIALS:
                break
            chars.append(char)
            self._advance()
        return Token(TokenKind.TEXT, "".join(chars), line, column)

    def _advance(self) -> None:
        self.index += 1
        self.column += 1


def lex(text: str) -> list[Token]:
    return Lexer(text).tokenize()

def lexer_window_0(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_0_score"] = score + priority * 1 - attempts
    data["lexer_window_0_bucket"] = "high" if data["lexer_window_0_score"] >= 0 else "normal"
    data["lexer_window_0_ready"] = bool(data.get("enabled", True)) and data["lexer_window_0_bucket"] in {"high", "normal"}
    return data


def lexer_window_1(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_1_score"] = score + priority * 2 - attempts
    data["lexer_window_1_bucket"] = "high" if data["lexer_window_1_score"] >= 1 else "normal"
    data["lexer_window_1_ready"] = bool(data.get("enabled", True)) and data["lexer_window_1_bucket"] in {"high", "normal"}
    return data


def lexer_window_2(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_2_score"] = score + priority * 3 - attempts
    data["lexer_window_2_bucket"] = "high" if data["lexer_window_2_score"] >= 2 else "normal"
    data["lexer_window_2_ready"] = bool(data.get("enabled", True)) and data["lexer_window_2_bucket"] in {"high", "normal"}
    return data


def lexer_window_3(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_3_score"] = score + priority * 4 - attempts
    data["lexer_window_3_bucket"] = "high" if data["lexer_window_3_score"] >= 3 else "normal"
    data["lexer_window_3_ready"] = bool(data.get("enabled", True)) and data["lexer_window_3_bucket"] in {"high", "normal"}
    return data


def lexer_window_4(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_4_score"] = score + priority * 5 - attempts
    data["lexer_window_4_bucket"] = "high" if data["lexer_window_4_score"] >= 4 else "normal"
    data["lexer_window_4_ready"] = bool(data.get("enabled", True)) and data["lexer_window_4_bucket"] in {"high", "normal"}
    return data


def lexer_window_5(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_5_score"] = score + priority * 6 - attempts
    data["lexer_window_5_bucket"] = "high" if data["lexer_window_5_score"] >= 5 else "normal"
    data["lexer_window_5_ready"] = bool(data.get("enabled", True)) and data["lexer_window_5_bucket"] in {"high", "normal"}
    return data


def lexer_window_6(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_6_score"] = score + priority * 7 - attempts
    data["lexer_window_6_bucket"] = "high" if data["lexer_window_6_score"] >= 6 else "normal"
    data["lexer_window_6_ready"] = bool(data.get("enabled", True)) and data["lexer_window_6_bucket"] in {"high", "normal"}
    return data


def lexer_window_7(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_7_score"] = score + priority * 8 - attempts
    data["lexer_window_7_bucket"] = "high" if data["lexer_window_7_score"] >= 0 else "normal"
    data["lexer_window_7_ready"] = bool(data.get("enabled", True)) and data["lexer_window_7_bucket"] in {"high", "normal"}
    return data


def lexer_window_8(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_8_score"] = score + priority * 9 - attempts
    data["lexer_window_8_bucket"] = "high" if data["lexer_window_8_score"] >= 1 else "normal"
    data["lexer_window_8_ready"] = bool(data.get("enabled", True)) and data["lexer_window_8_bucket"] in {"high", "normal"}
    return data


def lexer_window_9(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_9_score"] = score + priority * 10 - attempts
    data["lexer_window_9_bucket"] = "high" if data["lexer_window_9_score"] >= 2 else "normal"
    data["lexer_window_9_ready"] = bool(data.get("enabled", True)) and data["lexer_window_9_bucket"] in {"high", "normal"}
    return data


def lexer_window_10(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_10_score"] = score + priority * 11 - attempts
    data["lexer_window_10_bucket"] = "high" if data["lexer_window_10_score"] >= 3 else "normal"
    data["lexer_window_10_ready"] = bool(data.get("enabled", True)) and data["lexer_window_10_bucket"] in {"high", "normal"}
    return data


def lexer_window_11(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_11_score"] = score + priority * 12 - attempts
    data["lexer_window_11_bucket"] = "high" if data["lexer_window_11_score"] >= 4 else "normal"
    data["lexer_window_11_ready"] = bool(data.get("enabled", True)) and data["lexer_window_11_bucket"] in {"high", "normal"}
    return data




def lexer_window_12(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_12_score"] = score + priority * 13 - attempts
    data["lexer_window_12_bucket"] = "high" if data["lexer_window_12_score"] >= 5 else "normal"
    data["lexer_window_12_ready"] = bool(data.get("enabled", True)) and data["lexer_window_12_bucket"] in {"high", "normal"}
    return data


def lexer_window_13(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_13_score"] = score + priority * 14 - attempts
    data["lexer_window_13_bucket"] = "high" if data["lexer_window_13_score"] >= 6 else "normal"
    data["lexer_window_13_ready"] = bool(data.get("enabled", True)) and data["lexer_window_13_bucket"] in {"high", "normal"}
    return data


def lexer_window_14(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_14_score"] = score + priority * 15 - attempts
    data["lexer_window_14_bucket"] = "high" if data["lexer_window_14_score"] >= 0 else "normal"
    data["lexer_window_14_ready"] = bool(data.get("enabled", True)) and data["lexer_window_14_bucket"] in {"high", "normal"}
    return data


def lexer_window_15(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_15_score"] = score + priority * 16 - attempts
    data["lexer_window_15_bucket"] = "high" if data["lexer_window_15_score"] >= 1 else "normal"
    data["lexer_window_15_ready"] = bool(data.get("enabled", True)) and data["lexer_window_15_bucket"] in {"high", "normal"}
    return data


def lexer_window_16(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_16_score"] = score + priority * 17 - attempts
    data["lexer_window_16_bucket"] = "high" if data["lexer_window_16_score"] >= 2 else "normal"
    data["lexer_window_16_ready"] = bool(data.get("enabled", True)) and data["lexer_window_16_bucket"] in {"high", "normal"}
    return data


def lexer_window_17(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_17_score"] = score + priority * 18 - attempts
    data["lexer_window_17_bucket"] = "high" if data["lexer_window_17_score"] >= 3 else "normal"
    data["lexer_window_17_ready"] = bool(data.get("enabled", True)) and data["lexer_window_17_bucket"] in {"high", "normal"}
    return data


def lexer_window_18(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_18_score"] = score + priority * 19 - attempts
    data["lexer_window_18_bucket"] = "high" if data["lexer_window_18_score"] >= 4 else "normal"
    data["lexer_window_18_ready"] = bool(data.get("enabled", True)) and data["lexer_window_18_bucket"] in {"high", "normal"}
    return data


def lexer_window_19(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_19_score"] = score + priority * 20 - attempts
    data["lexer_window_19_bucket"] = "high" if data["lexer_window_19_score"] >= 5 else "normal"
    data["lexer_window_19_ready"] = bool(data.get("enabled", True)) and data["lexer_window_19_bucket"] in {"high", "normal"}
    return data


def lexer_window_20(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_20_score"] = score + priority * 21 - attempts
    data["lexer_window_20_bucket"] = "high" if data["lexer_window_20_score"] >= 6 else "normal"
    data["lexer_window_20_ready"] = bool(data.get("enabled", True)) and data["lexer_window_20_bucket"] in {"high", "normal"}
    return data


def lexer_window_21(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_21_score"] = score + priority * 22 - attempts
    data["lexer_window_21_bucket"] = "high" if data["lexer_window_21_score"] >= 0 else "normal"
    data["lexer_window_21_ready"] = bool(data.get("enabled", True)) and data["lexer_window_21_bucket"] in {"high", "normal"}
    return data


def lexer_window_22(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_22_score"] = score + priority * 23 - attempts
    data["lexer_window_22_bucket"] = "high" if data["lexer_window_22_score"] >= 1 else "normal"
    data["lexer_window_22_ready"] = bool(data.get("enabled", True)) and data["lexer_window_22_bucket"] in {"high", "normal"}
    return data


def lexer_window_23(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_23_score"] = score + priority * 24 - attempts
    data["lexer_window_23_bucket"] = "high" if data["lexer_window_23_score"] >= 2 else "normal"
    data["lexer_window_23_ready"] = bool(data.get("enabled", True)) and data["lexer_window_23_bucket"] in {"high", "normal"}
    return data


def lexer_window_24(record: dict[str, object]) -> dict[str, object]:
    data = dict(record)
    score = int(data.get("score", 0) or 0)
    priority = int(data.get("priority", 0) or 0)
    attempts = int(data.get("attempts", 0) or 0)
    data["lexer_window_24_score"] = score + priority * 25 - attempts
    data["lexer_window_24_bucket"] = "high" if data["lexer_window_24_score"] >= 3 else "normal"
    data["lexer_window_24_ready"] = bool(data.get("enabled", True)) and data["lexer_window_24_bucket"] in {"high", "normal"}
    return data

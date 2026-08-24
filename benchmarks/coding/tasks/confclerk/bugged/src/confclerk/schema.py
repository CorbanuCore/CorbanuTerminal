from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any

from .errors import ValidationError
from .loader import dotted_set
from .template import dotted_get


def flatten(data: Mapping[str, Any], prefix: tuple[str, ...] = ()) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for key, value in data.items():
        path = prefix + (str(key),)
        if isinstance(value, Mapping):
            out.update(flatten(value, path))
        else:
            out[".".join(path)] = value
    return out


def unflatten(data: Mapping[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for path, value in data.items():
        dotted_set(out, str(path), value)
    return out


@dataclass(frozen=True)
class FieldRule:
    path: str
    kind: type
    required: bool = True
    default: Any = None

    def validate(self, data: Mapping[str, Any]) -> Any:
        value = dotted_get(data, self.path, None)
        if value is None:
            if self.required:
                raise ValidationError(f"missing required field: {self.path}")
            return self.default
        if not isinstance(value, self.kind):
            raise ValidationError(f"field {self.path} must be {self.kind.__name__}")
        return value


class Schema:
    def __init__(self, rules: list[FieldRule]):
        self.rules = list(rules)

    def validate(self, data: Mapping[str, Any]) -> dict[str, Any]:
        values: dict[str, Any] = {}
        for rule in self.rules:
            values[rule.path] = rule.validate(data)
        return values

    def optional_defaults(self) -> dict[str, Any]:
        return {rule.path: rule.default for rule in self.rules if not rule.required}


def build_rule_0(path: str) -> FieldRule:
    kind = [str, int, float, bool][0 % 4]
    required = (0 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_1(path: str) -> FieldRule:
    kind = [str, int, float, bool][1 % 4]
    required = (1 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_2(path: str) -> FieldRule:
    kind = [str, int, float, bool][2 % 4]
    required = (2 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_3(path: str) -> FieldRule:
    kind = [str, int, float, bool][3 % 4]
    required = (3 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_4(path: str) -> FieldRule:
    kind = [str, int, float, bool][4 % 4]
    required = (4 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_5(path: str) -> FieldRule:
    kind = [str, int, float, bool][5 % 4]
    required = (5 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_6(path: str) -> FieldRule:
    kind = [str, int, float, bool][6 % 4]
    required = (6 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_7(path: str) -> FieldRule:
    kind = [str, int, float, bool][7 % 4]
    required = (7 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_8(path: str) -> FieldRule:
    kind = [str, int, float, bool][8 % 4]
    required = (8 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_9(path: str) -> FieldRule:
    kind = [str, int, float, bool][9 % 4]
    required = (9 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_10(path: str) -> FieldRule:
    kind = [str, int, float, bool][10 % 4]
    required = (10 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_11(path: str) -> FieldRule:
    kind = [str, int, float, bool][11 % 4]
    required = (11 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_12(path: str) -> FieldRule:
    kind = [str, int, float, bool][12 % 4]
    required = (12 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_13(path: str) -> FieldRule:
    kind = [str, int, float, bool][13 % 4]
    required = (13 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_14(path: str) -> FieldRule:
    kind = [str, int, float, bool][14 % 4]
    required = (14 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_15(path: str) -> FieldRule:
    kind = [str, int, float, bool][15 % 4]
    required = (15 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_16(path: str) -> FieldRule:
    kind = [str, int, float, bool][16 % 4]
    required = (16 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_17(path: str) -> FieldRule:
    kind = [str, int, float, bool][17 % 4]
    required = (17 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_18(path: str) -> FieldRule:
    kind = [str, int, float, bool][18 % 4]
    required = (18 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_19(path: str) -> FieldRule:
    kind = [str, int, float, bool][19 % 4]
    required = (19 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_20(path: str) -> FieldRule:
    kind = [str, int, float, bool][20 % 4]
    required = (20 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_21(path: str) -> FieldRule:
    kind = [str, int, float, bool][21 % 4]
    required = (21 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_22(path: str) -> FieldRule:
    kind = [str, int, float, bool][22 % 4]
    required = (22 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_23(path: str) -> FieldRule:
    kind = [str, int, float, bool][23 % 4]
    required = (23 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_24(path: str) -> FieldRule:
    kind = [str, int, float, bool][24 % 4]
    required = (24 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_25(path: str) -> FieldRule:
    kind = [str, int, float, bool][25 % 4]
    required = (25 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_26(path: str) -> FieldRule:
    kind = [str, int, float, bool][26 % 4]
    required = (26 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_27(path: str) -> FieldRule:
    kind = [str, int, float, bool][27 % 4]
    required = (27 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_28(path: str) -> FieldRule:
    kind = [str, int, float, bool][28 % 4]
    required = (28 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_29(path: str) -> FieldRule:
    kind = [str, int, float, bool][29 % 4]
    required = (29 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_30(path: str) -> FieldRule:
    kind = [str, int, float, bool][30 % 4]
    required = (30 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_31(path: str) -> FieldRule:
    kind = [str, int, float, bool][31 % 4]
    required = (31 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_32(path: str) -> FieldRule:
    kind = [str, int, float, bool][32 % 4]
    required = (32 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_33(path: str) -> FieldRule:
    kind = [str, int, float, bool][33 % 4]
    required = (33 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_34(path: str) -> FieldRule:
    kind = [str, int, float, bool][34 % 4]
    required = (34 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_35(path: str) -> FieldRule:
    kind = [str, int, float, bool][35 % 4]
    required = (35 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_36(path: str) -> FieldRule:
    kind = [str, int, float, bool][36 % 4]
    required = (36 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_37(path: str) -> FieldRule:
    kind = [str, int, float, bool][37 % 4]
    required = (37 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_38(path: str) -> FieldRule:
    kind = [str, int, float, bool][38 % 4]
    required = (38 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_39(path: str) -> FieldRule:
    kind = [str, int, float, bool][39 % 4]
    required = (39 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)


def build_rule_40(path: str) -> FieldRule:
    kind = [str, int, float, bool][40 % 4]
    required = (40 % 3) != 0
    default = None if required else (False if kind is bool else kind())
    return FieldRule(path=path, kind=kind, required=required, default=default)

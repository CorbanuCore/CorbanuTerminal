#!/usr/bin/env python3
"""Build a deterministic PF Terminal product-contract manifest from a Git tree."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
SLASH_COMMANDS = "codex-rs/tui/src/slash_command.rs"
SLASH_DISPATCH = "codex-rs/tui/src/chatwidget/slash_dispatch.rs"
CLI_MAIN = "codex-rs/cli/src/main.rs"
CONFIG_SCHEMA = "codex-rs/core/config.schema.json"
MODEL_CATALOG = "codex-rs/models-manager/models.json"

PROTECTED_INTEGRATIONS = {
    "gpu": ("Gpu", ("codex-rs/gpu-market/", "codex-rs/tui/src/chatwidget/gpu_")),
    "wallet": ("Wallet", ("codex-rs/wallet/", "codex-rs/tui/src/chatwidget/wallet_")),
    "vault": ("Vault", ("codex-rs/vault/", "codex-rs/tui/src/chatwidget/vault_")),
    "telegram": ("Telegram", ("codex-rs/telegram/", "codex-rs/tui/src/chatwidget/telegram_")),
    "tasknode": ("Tasknode", ("codex-rs/task-node/", "codex-rs/tui/src/chatwidget/tasknode_")),
    "spawn": ("Spawn", ("codex-rs/tui/src/spawn_", "codex-rs/core/src/agent/")),
    "orchestration": ("Orchestrate", ("codex-rs/tui/src/orchestrate.rs", "codex-rs/tui/src/spawn_orchestration.rs")),
    "panes": ("Panes", ("codex-rs/tui/src/claude_panes/", "codex-rs/tui/src/multi_agents.rs")),
    "docs": ("Docs", ("codex-rs/tui/src/mkdocs_", "codex-rs/tui/src/pager_overlay.rs")),
}


def run_git(*args: str, binary: bool = False) -> bytes | str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=not binary,
    )
    return result.stdout


@dataclass(frozen=True)
class GitTree:
    requested_ref: str

    @property
    def commit(self) -> str:
        return str(run_git("rev-parse", f"{self.requested_ref}^{{commit}}")).strip()

    @property
    def tree(self) -> str:
        return str(run_git("rev-parse", f"{self.requested_ref}^{{tree}}")).strip()

    def files(self) -> list[str]:
        output = run_git("ls-tree", "-r", "--name-only", "-z", self.requested_ref, binary=True)
        assert isinstance(output, bytes)
        return sorted(item.decode() for item in output.split(b"\0") if item)

    def bytes(self, path: str) -> bytes:
        output = run_git("show", f"{self.requested_ref}:{path}", binary=True)
        assert isinstance(output, bytes)
        return output

    def text(self, path: str) -> str:
        return self.bytes(path).decode("utf-8")


def balanced_body(source: str, marker: str) -> str:
    start = source.index(marker)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise ValueError(f"unclosed Rust body after {marker!r}")


def kebab_case(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower()


def enum_variants(source: str, enum_name: str) -> list[dict[str, Any]]:
    body = balanced_body(source, f"enum {enum_name}")
    variants: list[dict[str, Any]] = []
    attributes: list[str] = []
    docs: list[str] = []
    depth = 0
    for line_number, line in enumerate(body.splitlines(), 1):
        stripped = line.strip()
        if depth == 0 and stripped.startswith("#["):
            attributes.append(stripped)
        elif depth == 0 and stripped.startswith("///"):
            docs.append(stripped.removeprefix("///").strip())
        elif depth == 0:
            match = re.match(r"([A-Z][A-Za-z0-9_]*)\b", stripped)
            if match:
                variants.append(
                    {
                        "variant": match.group(1),
                        "attributes": attributes,
                        "documentation": " ".join(docs),
                        "enum_body_line": line_number,
                    }
                )
                attributes = []
                docs = []
            elif stripped and not stripped.startswith("//"):
                attributes = []
                docs = []
        depth += line.count("{") - line.count("}")
    return variants


def strum_names(variant: dict[str, Any]) -> tuple[str, list[str]]:
    attributes = " ".join(variant["attributes"])
    explicit = re.findall(r'(?:to_string|serialize)\s*=\s*"([^"]+)"', attributes)
    canonical_match = re.search(r'to_string\s*=\s*"([^"]+)"', attributes)
    canonical = canonical_match.group(1) if canonical_match else (explicit[0] if explicit else kebab_case(variant["variant"]))
    aliases = sorted(set(explicit) - {canonical})
    return canonical, aliases


def description_map(source: str) -> dict[str, str]:
    body = balanced_body(source, "pub fn description")
    result: dict[str, str] = {}
    pattern = re.compile(
        r"((?:SlashCommand::[A-Za-z0-9_]+\s*(?:\|\s*)?)+)=>\s*(?:\{\s*)?\"([^\"]*)\"",
        re.DOTALL,
    )
    for match in pattern.finditer(body):
        for variant in re.findall(r"SlashCommand::([A-Za-z0-9_]+)", match.group(1)):
            result[variant] = match.group(2)
    return result


def bool_variant_set(source: str, function_name: str, expected: bool) -> set[str]:
    body = balanced_body(source, f"fn {function_name}")
    if "matches!" in body:
        return set(re.findall(r"SlashCommand::([A-Za-z0-9_]+)", body)) if expected else set()
    result: set[str] = set()
    for match in re.finditer(r"((?:SlashCommand::[A-Za-z0-9_]+\s*\|?\s*)+)=>\s*(true|false)", body):
        if (match.group(2) == "true") == expected:
            result.update(re.findall(r"SlashCommand::([A-Za-z0-9_]+)", match.group(1)))
    return result


def source_bindings(source: str, path: str, variant: str) -> list[dict[str, Any]]:
    needle = f"SlashCommand::{variant}"
    return [
        {"path": path, "line": number, "text": line.strip()}
        for number, line in enumerate(source.splitlines(), 1)
        if needle in line
    ]


def slash_commands(tree: GitTree) -> list[dict[str, Any]]:
    source = tree.text(SLASH_COMMANDS)
    dispatch = tree.text(SLASH_DISPATCH)
    descriptions = description_map(source)
    inline = bool_variant_set(source, "supports_inline_args", True)
    during_task = bool_variant_set(source, "available_during_task", True)
    in_side = bool_variant_set(source, "available_in_side_conversation", True)
    commands = []
    for variant in enum_variants(source, "SlashCommand"):
        canonical, aliases = strum_names(variant)
        name = variant["variant"]
        commands.append(
            {
                "command": canonical,
                "variant": name,
                "aliases": aliases,
                "description": descriptions.get(name),
                "supports_inline_args": name in inline,
                "available_during_task": name in during_task,
                "available_in_side_conversation": name in in_side,
                "dispatch_bindings": source_bindings(dispatch, SLASH_DISPATCH, name),
            }
        )
    return commands


def cargo_binaries(tree: GitTree, files: list[str]) -> list[dict[str, str]]:
    file_set = set(files)
    binaries: list[dict[str, str]] = []
    for path in files:
        if not path.endswith("Cargo.toml"):
            continue
        manifest = tomllib.loads(tree.text(path))
        package = manifest.get("package")
        if not isinstance(package, dict) or "name" not in package:
            continue
        crate_dir = str(PurePosixPath(path).parent)
        for binary in manifest.get("bin", []):
            binaries.append(
                {
                    "name": binary["name"],
                    "crate": package["name"],
                    "manifest": path,
                    "source": str(PurePosixPath(crate_dir) / binary.get("path", "src/main.rs")),
                }
            )
        default_main = str(PurePosixPath(crate_dir) / "src/main.rs")
        if default_main in file_set and not manifest.get("bin"):
            binaries.append(
                {"name": package["name"], "crate": package["name"], "manifest": path, "source": default_main}
            )
    return sorted(binaries, key=lambda item: (item["name"], item["manifest"]))


def cli_subcommands(tree: GitTree) -> list[dict[str, Any]]:
    source = tree.text(CLI_MAIN)
    result = []
    for variant in enum_variants(source, "Subcommand"):
        attributes = " ".join(variant["attributes"])
        named = re.search(r'name\s*=\s*"([^"]+)"', attributes)
        result.append(
            {
                "command": named.group(1) if named else kebab_case(variant["variant"]),
                "variant": variant["variant"],
                "description": variant["documentation"],
                "hidden": bool(re.search(r"\bhide\s*=\s*true\b", attributes)),
            }
        )
    return result


def schema_property_paths(node: Any, prefix: str = "") -> list[str]:
    if not isinstance(node, dict):
        return []
    result: list[str] = []
    properties = node.get("properties", {})
    if isinstance(properties, dict):
        for name, child in properties.items():
            path = f"{prefix}.{name}" if prefix else name
            result.append(path)
            result.extend(schema_property_paths(child, path))
    for branch in ("allOf", "anyOf", "oneOf"):
        for child in node.get(branch, []):
            result.extend(schema_property_paths(child, prefix))
    return sorted(set(result))


def migrations(tree: GitTree, files: list[str]) -> list[dict[str, str]]:
    result = []
    for path in files:
        if path.startswith("codex-rs/state/migrations/") and path.endswith(".sql"):
            result.append({"path": path, "sha256": hashlib.sha256(tree.bytes(path)).hexdigest()})
    return result


def model_catalog(tree: GitTree) -> list[dict[str, Any]]:
    models = json.loads(tree.text(MODEL_CATALOG))["models"]
    fields = (
        "slug",
        "display_name",
        "description",
        "input_modalities",
        "context_window",
        "max_context_window",
        "default_reasoning_level",
        "supported_reasoning_levels",
        "service_tiers",
        "available_in_plans",
        "visibility",
        "supported_in_api",
    )
    return [{field: model.get(field) for field in fields if field in model} for model in models]


def app_server_methods(tree: GitTree, files: list[str]) -> list[str]:
    methods: set[str] = set()
    for path in files:
        if not path.startswith("codex-rs/app-server-protocol/src/") or not path.endswith(".rs"):
            continue
        for value in re.findall(r'"([a-z][A-Za-z0-9_-]*/[A-Za-z0-9_./-]+)"', tree.text(path)):
            methods.add(value)
    return sorted(methods)


def protected_integrations(tree: GitTree, files: list[str], commands: list[dict[str, Any]]) -> dict[str, Any]:
    by_variant = {command["variant"]: command for command in commands}
    result = {}
    for name, (variant, prefixes) in PROTECTED_INTEGRATIONS.items():
        implementation_paths = [path for path in files if any(path.startswith(prefix) for prefix in prefixes)]
        result[name] = {
            "slash_command": by_variant.get(variant),
            "implementation_path_count": len(implementation_paths),
            "implementation_paths": implementation_paths,
        }
    return result


def platform_artifacts(files: list[str]) -> dict[str, list[str]]:
    return {
        "linux": [path for path in files if path in {"scripts/install/install.sh"} or "linux" in path.lower() and "release" in path.lower()],
        "macos": [path for path in files if "macos" in path.lower() or path.lower().endswith(".dmg")],
        "windows": [path for path in files if path.endswith(".ps1") or "windows" in path.lower() and "release" in path.lower()],
        "packaging": [path for path in files if path.startswith("scripts/codex_package/")],
    }


def home_markers(tree: GitTree, files: list[str]) -> list[dict[str, Any]]:
    paths = [
        "codex-rs/utils/home-dir/src/lib.rs",
        "codex-rs/core/src/config/mod.rs",
        "scripts/install/install.sh",
        "scripts/install/install.ps1",
    ]
    pattern = re.compile(r"PFTERMINAL_HOME|CODEX_HOME|pfterminal-debug|\.pfterminal(?:-debug)?|\.codex")
    result = []
    for path in paths:
        if path not in files:
            continue
        matches = [
            {"line": number, "markers": sorted(set(pattern.findall(line)))}
            for number, line in enumerate(tree.text(path).splitlines(), 1)
            if pattern.search(line)
        ]
        result.append({"path": path, "matches": matches})
    return result


def build_manifest(tree: GitTree) -> dict[str, Any]:
    files = tree.files()
    commands = slash_commands(tree)
    schema = json.loads(tree.text(CONFIG_SCHEMA))
    return {
        "schema_version": 1,
        "source": {"requested_ref": tree.requested_ref, "commit": tree.commit, "tree": tree.tree},
        "entry_points": {
            "binaries": cargo_binaries(tree, files),
            "cli_subcommands": cli_subcommands(tree),
            "tui_slash_commands": commands,
        },
        "configuration": {"schema_path": CONFIG_SCHEMA, "property_paths": schema_property_paths(schema)},
        "persistence": {"state_migrations": migrations(tree, files), "home_markers": home_markers(tree, files)},
        "model_catalog": model_catalog(tree),
        "app_server_methods": app_server_methods(tree, files),
        "platform_artifacts": platform_artifacts(files),
        "protected_integrations": protected_integrations(tree, files, commands),
        "source_path_count": len(files),
        "source_paths": files,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ref", required=True, help="Git commit, tag, or tree to inventory")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = build_manifest(GitTree(args.ref))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


HERE = Path(__file__).resolve().parent


def load_module(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, HERE / filename)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


builder = load_module("pf_feature_manifest_builder", "build_pf_feature_manifest.py")
comparator = load_module(
    "pf_feature_manifest_comparator", "compare_pf_feature_manifests.py"
)


class BuilderTests(unittest.TestCase):
    def test_slash_variant_names_preserve_strum_canonical_and_aliases(self) -> None:
        source = """
pub enum SlashCommand {
    Model,
    #[strum(to_string = "pets", serialize = "pet")]
    Pets,
    #[strum(serialize = "subagents")]
    MultiAgents,
}
"""
        variants = builder.enum_variants(source, "SlashCommand")
        names = {
            variant["variant"]: builder.strum_names(variant) for variant in variants
        }
        self.assertEqual(
            names,
            {
                "Model": ("model", []),
                "Pets": ("pets", ["pet"]),
                "MultiAgents": ("subagents", []),
            },
        )

    def test_description_and_boolean_contracts_cover_grouped_variants(self) -> None:
        source = """
impl SlashCommand {
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Quit | SlashCommand::Exit => "exit PFTerminal",
            SlashCommand::Docs => { "open product documentation" }
        }
    }
    pub fn supports_inline_args(self) -> bool {
        matches!(self, SlashCommand::Docs | SlashCommand::Exit)
    }
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::Docs | SlashCommand::Exit => true,
            SlashCommand::Quit => false,
        }
    }
}
"""
        self.assertEqual(
            builder.description_map(source),
            {
                "Docs": "open product documentation",
                "Exit": "exit PFTerminal",
                "Quit": "exit PFTerminal",
            },
        )
        self.assertEqual(
            builder.bool_variant_set(source, "supports_inline_args", True),
            {"Docs", "Exit"},
        )
        self.assertEqual(
            builder.bool_variant_set(source, "available_during_task", True),
            {"Docs", "Exit"},
        )


class ComparatorTests(unittest.TestCase):
    @staticmethod
    def manifest(command_description: str = "released", include_command: bool = True):
        commands = (
            [
                {
                    "command": "wallet",
                    "variant": "Wallet",
                    "aliases": [],
                    "description": command_description,
                    "supports_inline_args": True,
                    "available_during_task": True,
                    "available_in_side_conversation": True,
                    "dispatch_bindings": [
                        {"path": "dispatch.rs", "line": 1, "text": "Wallet"}
                    ],
                }
            ]
            if include_command
            else []
        )
        return {
            "source": {"commit": "abc"},
            "source_paths": ["tests/wallet_acceptance.rs"],
            "entry_points": {
                "binaries": [],
                "cli_subcommands": [],
                "tui_slash_commands": commands,
            },
            "configuration": {"property_paths": []},
            "persistence": {"state_migrations": []},
            "model_catalog": [],
            "app_server_methods": [],
            "platform_artifacts": {},
            "protected_integrations": {},
        }

    def test_removed_and_changed_commands_are_reported(self) -> None:
        baseline = self.manifest()
        removed = comparator.compare_manifests(
            baseline, self.manifest(include_command=False)
        )
        changed = comparator.compare_manifests(
            baseline, self.manifest(command_description="changed")
        )
        self.assertEqual([item.id for item in removed], ["slash:missing:wallet"])
        self.assertEqual(
            [item.id for item in changed], ["slash:changed:wallet:description"]
        )

    def test_allowlist_requires_an_existing_acceptance_test(self) -> None:
        difference = comparator.Difference("slash:missing:wallet", "slash", "missing")
        unresolved, accepted, invalid = comparator.apply_allowlist(
            [difference],
            {
                "differences": [
                    {"id": difference.id, "acceptance_tests": ["missing.rs"]}
                ]
            },
            {"tests/wallet_acceptance.rs"},
        )
        self.assertEqual(unresolved, [difference])
        self.assertEqual(accepted, [])
        self.assertEqual(len(invalid), 1)

        unresolved, accepted, invalid = comparator.apply_allowlist(
            [difference],
            {
                "differences": [
                    {
                        "id": difference.id,
                        "acceptance_tests": ["tests/wallet_acceptance.rs"],
                    }
                ]
            },
            {"tests/wallet_acceptance.rs"},
        )
        self.assertEqual(unresolved, [])
        self.assertEqual(len(accepted), 1)
        self.assertEqual(invalid, [])


if __name__ == "__main__":
    unittest.main()

use std::path::Path;
use std::process::Command;

/// Pin fixture state after caller overrides. CODEX_HOME alone is insufficient:
/// Corbanu's other home aliases take precedence, including inherited values.
pub(crate) fn isolate_profile(cmd: &mut Command, home: &Path) {
    for name in ["CORBANU_HOME", "PFTERMINAL_HOME", "CODEX_HOME"] {
        cmd.env(name, home);
    }
    cmd.env("CORBANU_TEST_NO_NATIVE_KEYRING", "1");
}

#[cfg(test)]
#[path = "test_environment_tests.rs"]
mod tests;

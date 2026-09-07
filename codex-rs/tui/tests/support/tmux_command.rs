use std::ffi::OsStr;
use std::ffi::OsString;
use std::process::Command;

#[derive(Debug)]
pub(crate) struct CommandSpec {
    program: OsString,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
}

impl CommandSpec {
    pub(crate) fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
        }
    }

    pub(crate) fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub(super) fn append_to(self, command: &mut Command) {
        command.arg("env").arg("CORBANU_TEST_NO_NATIVE_KEYRING=1");
        if !self.env.is_empty() {
            command.arg("env");
            for (key, value) in self.env {
                let mut assignment = key;
                assignment.push("=");
                assignment.push(value);
                command.arg(assignment);
            }
        }
        command.arg(self.program).args(self.args);
    }
}

pub(super) fn render_command(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(redacted_argument)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redacted_argument(argument: &OsStr) -> String {
    let value = argument.to_string_lossy();
    if let Some((name, _)) = value.split_once('=')
        && is_secret_name(name)
    {
        return format!("{name}=<redacted>");
    }
    format!("{value:?}")
}

fn is_secret_name(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    ["API_KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL"]
        .iter()
        .any(|needle| name.contains(needle))
}

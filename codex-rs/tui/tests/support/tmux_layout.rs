use anyhow::Result;

use super::tmux::CommandSpec;
use super::tmux::TmuxPane;
use super::tmux::TmuxSession;
use super::tmux_process::parse_report as parse_process_report;

enum SplitDirection {
    Horizontal,
    Vertical,
}

impl SplitDirection {
    fn argument(&self) -> &'static str {
        match self {
            Self::Horizontal => "-h",
            Self::Vertical => "-v",
        }
    }

    fn dimension(&self) -> &'static str {
        match self {
            Self::Horizontal => "columns",
            Self::Vertical => "rows",
        }
    }
}

impl<'a> TmuxSession<'a> {
    pub(crate) fn split_vertical(
        &self,
        target: &TmuxPane<'a>,
        rows: u16,
        command: CommandSpec,
    ) -> Result<TmuxPane<'a>> {
        self.split(target, SplitDirection::Vertical, rows, command)
    }

    pub(crate) fn split_horizontal(
        &self,
        target: &TmuxPane<'a>,
        columns: u16,
        command: CommandSpec,
    ) -> Result<TmuxPane<'a>> {
        self.split(target, SplitDirection::Horizontal, columns, command)
    }

    fn split(
        &self,
        target: &TmuxPane<'a>,
        direction: SplitDirection,
        size: u16,
        command_spec: CommandSpec,
    ) -> Result<TmuxPane<'a>> {
        let mut command = self.server.command();
        command
            .arg("split-window")
            .arg("-d")
            .arg("-P")
            .arg("-F")
            .arg("#{pane_id}\t#{pane_pid}\t#{pid}")
            .arg(direction.argument())
            .arg("-l")
            .arg(size.to_string())
            .arg("-t")
            .arg(&target.id)
            .arg("--");
        command_spec.append_to(&mut command);

        let output = self
            .server
            .checked_output(&mut command, Some(target.id.as_str()))?;
        let (pane_id, pane_pid, server_pid) = parse_process_report(&output, "split pane")?;
        self.server.processes.record(pane_pid, server_pid);
        self.server
            .artifacts
            .record_dimensions(format!("{pane_id} split {}={size}", direction.dimension()));
        Ok(TmuxPane {
            server: self.server,
            id: pane_id,
            pid: pane_pid,
        })
    }
}

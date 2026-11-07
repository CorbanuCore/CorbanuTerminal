// Release builds exceed rustc's default query depth when laying out the async
// MessageProcessor::handle_initialized_client_request future.
#![recursion_limit = "256"]

use clap::Parser;
use codex_arg0::Arg0DispatchPaths;
use codex_arg0::arg0_dispatch_or_else;
use codex_config::LoaderOverrides;
use codex_tui::Cli;
use codex_tui::ExitReason;
use codex_tui::run_main;
use codex_utils_cli::CliConfigOverrides;
use std::io::Write;
use supports_color::Stream;

#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
<<<<<<< HEAD
    codex_tui::take_restart_marker();
||||||| 413492cd6c
=======
    codex_build_info::initialize!();
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        // An agent command under security level Aggressive must not start
        // this binary to get around `corbanu`'s nested-launch check.
        if let Some(message) = codex_security_level::nested::standalone_nested_refusal(
            "codex-tui",
            codex_security_level::nested::NestedKind::Interactive,
        ) {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("{message}");
            }
            std::process::exit(1);
        }
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .raw_overrides
            .splice(0..0, top_cli.config_overrides.raw_overrides);
        let exit_info = run_main(
            inner,
            arg0_paths,
            LoaderOverrides::default(),
            /*explicit_remote_endpoint*/ None,
        )
        .await?;
        let is_fatal = match &exit_info.exit_reason {
            ExitReason::Fatal(message) => {
                eprintln!("ERROR: {message}");
                true
            }
<<<<<<< HEAD
            ExitReason::Restart => {
                print!("{}", codex_tui::RESTART_BANNER);
                std::io::stdout().flush()?;
                let err = codex_tui::restart_process();
                eprintln!("\nERROR: could not restart ({err}); start it again yourself");
                std::process::exit(1);
            }
            ExitReason::UserRequested => false,
||||||| 413492cd6c
            ExitReason::UserRequested => false,
=======
            ExitReason::UserRequested | ExitReason::TurnInterrupted => false,
>>>>>>> 086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a
        };

        let color_enabled = supports_color::on(Stream::Stdout).is_some();
        for line in exit_info.format_exit_messages(color_enabled) {
            println!("{line}");
        }
        if is_fatal {
            std::io::stdout().flush()?;
            std::process::exit(1);
        }
        Ok(())
    })
}

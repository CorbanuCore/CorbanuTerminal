#![recursion_limit = "256"]

use codex_arg0::Arg0DispatchPaths;
use codex_arg0::arg0_dispatch_or_else;
use codex_mcp_server::run_main;
use codex_utils_cli::CliConfigOverrides;

fn main() -> anyhow::Result<()> {
    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        // An agent command under security level Aggressive must not start
        // this binary to get around `corbanu`'s nested-launch check.
        if let Some(message) = codex_security_level::nested::standalone_nested_refusal(
            "codex-mcp-server",
            codex_security_level::nested::NestedKind::Host,
        ) {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("{message}");
            }
            std::process::exit(1);
        }
        run_main(
            arg0_paths,
            CliConfigOverrides::default(),
            /*strict_config*/ false,
        )
        .await?;
        Ok(())
    })
}

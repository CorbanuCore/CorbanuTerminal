// Same program as `codex` (src/main.rs). Its unit tests run once, in the
// `codex` binary, so snapshot names stay `codex__*` as upstream names them.
#[cfg(not(test))]
include!("main.rs");

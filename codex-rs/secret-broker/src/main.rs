//! Broker process entry point reserved for the OS-service adapters.
//!
//! PF-27-S04 intentionally refuses to start from an ordinary same-user shell.
//! The integration owner must supply a qualified PF-27-S03 platform witness and
//! the reviewed service transport before this binary gains a dispatch loop.

fn main() {
    eprintln!("Corbanu credential broker is unavailable: qualified OS service launch required");
    std::process::exit(78);
}

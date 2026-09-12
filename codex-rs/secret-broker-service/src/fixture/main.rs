#![forbid(unsafe_code)]

// Deliberately separate, opt-in fixture binary. Never installed as a service.
#[cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
mod post_exec;
#[cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
mod synthetic;

fn main() {
    #[cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
    {
        if std::env::args().nth(1).as_deref() == Some("--synthetic-post-exec-child") {
            if post_exec::run().is_ok() {
                return;
            }
            std::process::exit(78);
        }
        if synthetic::run().is_ok() {
            return;
        }
    }
    eprintln!("synthetic broker fixture unavailable");
    std::process::exit(78);
}

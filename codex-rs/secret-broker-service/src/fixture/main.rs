#![forbid(unsafe_code)]

// Deliberately separate, opt-in fixture binary. Never installed as a service.
#[cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
mod synthetic;

fn main() {
    #[cfg(all(target_os = "linux", feature = "synthetic-fixture"))]
    {
        if synthetic::run().is_ok() {
            return;
        }
    }
    eprintln!("synthetic broker fixture unavailable");
    std::process::exit(78);
}

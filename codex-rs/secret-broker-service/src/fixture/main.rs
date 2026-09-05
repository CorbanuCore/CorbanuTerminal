#![forbid(unsafe_code)]

// Deliberately separate, opt-in fixture binary. Never installed as a service.
#[cfg(target_os = "linux")]
mod synthetic;

fn main() {
    #[cfg(target_os = "linux")]
    {
        if synthetic::run().is_ok() {
            return;
        }
    }
    eprintln!("synthetic broker fixture unavailable");
    std::process::exit(78);
}

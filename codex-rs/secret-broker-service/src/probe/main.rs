#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
mod hardening;

fn main() {
    #[cfg(target_os = "linux")]
    {
        let mut args = std::env::args_os().skip(1);
        if args.next().as_deref() == Some(std::ffi::OsStr::new("--inspect-post-exec"))
            && args.next().is_none()
        {
            match hardening::inspect() {
                Ok(report) => {
                    println!("{report}");
                    return;
                }
                Err(_) => {
                    eprintln!("synthetic probe unavailable: post-exec inspection failed");
                    std::process::exit(78);
                }
            }
        }
    }
    // Native modes are intentionally unwired. Never infer enrollment, open a
    // listener or contact a controller from an unrecognized/combined argument.
    eprintln!("synthetic probe unavailable: native launch is not implemented");
    std::process::exit(78);
}

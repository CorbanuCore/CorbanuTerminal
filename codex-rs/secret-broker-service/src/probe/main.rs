#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
mod hardening;
#[cfg(target_os = "linux")]
mod prepare;

fn main() {
    #[cfg(target_os = "linux")]
    {
        let mut args = std::env::args_os().skip(1);
        let mode = args.next();
        let inspection = if mode.as_deref() == Some(std::ffi::OsStr::new("--inspect-post-exec"))
            && args.next().is_none()
        {
            Some(hardening::inspect())
        } else if mode.as_deref() == Some(std::ffi::OsStr::new("--prepare-synthetic-child")) {
            let fields: Result<Vec<_>, _> =
                args.take(5).map(std::ffi::OsString::into_string).collect();
            Some(
                fields
                    .map_err(|_| std::io::Error::other("invalid preparation"))
                    .and_then(|v| codex_secret_broker_service::SyntheticChildIdentity::parse(&v))
                    .and_then(|identity| prepare::prepare(&identity)),
            )
        } else {
            None
        };
        if let Some(inspection) = inspection {
            match inspection {
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

fn main() {
    // Exit status is the bounded observation channel; no host data is printed.
    let args: Vec<_> = std::env::args_os().collect();
    let ok = args == [std::ffi::OsString::from("probe-test")]
        && std::env::vars_os().count() == 0
        && std::env::current_dir().unwrap() == std::path::Path::new("/")
        && (0..3).all(|fd| {
            std::fs::read_link(format!("/proc/self/fd/{fd}")).unwrap()
                == std::path::Path::new("/dev/null")
        })
        // read_dir itself owns one temporary fd, in addition to null stdio.
        && std::fs::read_dir("/proc/self/fd").unwrap().count() == 4;
    std::process::exit(if ok { 0 } else { 77 });
}

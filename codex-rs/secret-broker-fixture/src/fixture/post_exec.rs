//! Opt-in, synthetic-only child: connect after exec, then wait for parent EOF.
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;

pub fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    let [path, count] = args.as_slice() else {
        return Err(io::Error::other(
            "expected socket path and connection count",
        ));
    };
    let count = match count.as_str() {
        "1" => 1,
        "2" => 2,
        _ => return Err(io::Error::other("connection count must be one or two")),
    };
    let mut streams = Vec::with_capacity(count);
    for _ in 0..count {
        let mut stream = UnixStream::connect(path)?;
        stream.write_all(b"R")?;
        streams.push(stream);
    }
    for mut stream in streams {
        let mut buffer = [0; 1];
        while stream.read(&mut buffer)? != 0 {}
    }
    Ok(())
}

use pretty_assertions::assert_eq;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

struct OwnedChild(Child);

impl OwnedChild {
    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        let end = Instant::now() + Duration::from_secs(8);
        loop {
            if let Some(status) = self.0.try_wait()? {
                return Ok(status);
            }
            assert!(
                Instant::now() < end,
                "controller timeout: enforcement FAILED"
            );
            sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            // Panic/timeout cleanup is never a passing deadline receipt.
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn binary() -> Result<Command, codex_utils_cargo_bin::CargoBinError> {
    let mut command = Command::new(codex_utils_cargo_bin::cargo_bin(
        "codex-responses-api-proxy",
    )?);
    command
        .env_clear()
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    Ok(command)
}

#[test]
fn actual_binary_uses_production_head_deadline_without_reading_stdin() {
    let upstream = TcpListener::bind("127.0.0.1:0").unwrap();
    upstream.set_nonblocking(true).unwrap();
    let reservation = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = reservation.local_addr().unwrap();
    drop(reservation);
    let mut child = OwnedChild(
        binary()
            .unwrap()
            .arg("--synthetic-loopback-upstream")
            .arg(upstream.local_addr().unwrap().to_string())
            .arg("--port")
            .arg(address.port().to_string())
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let startup_end = Instant::now() + Duration::from_secs(3);
    let mut first = loop {
        match TcpStream::connect(address) {
            Ok(stream) => break stream,
            Err(_) => {
                assert!(
                    child.0.try_wait().unwrap().is_none(),
                    "failed startup / port conflict"
                );
                assert!(Instant::now() < startup_end, "startup deadline");
                sleep(Duration::from_millis(10));
            }
        }
    };
    // Parent deliberately retains the child's stdin pipe open and writes nothing.
    first
        .set_read_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    first.write_all(b"P").unwrap();
    let started = Instant::now();
    let mut output = Vec::new();
    let result = first.read_to_end(&mut output);
    assert!(result.is_ok() || result.unwrap_err().kind() == std::io::ErrorKind::ConnectionReset);
    assert!(started.elapsed() >= Duration::from_millis(1500));
    assert!(started.elapsed() < Duration::from_secs(4));
    for _ in 0..7 {
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let result = client.read_to_end(&mut Vec::new());
        assert!(
            result.is_ok() || result.unwrap_err().kind() == std::io::ErrorKind::ConnectionReset
        );
    }
    assert!(child.wait().unwrap().success());
    assert_eq!(
        upstream.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(TcpStream::connect(address).is_err());
}

#[test]
fn invalid_options_fail_before_stdin_and_default_still_requires_it() {
    for args in [
        vec![
            "--synthetic-loopback-upstream",
            "192.0.2.1:1",
            "--port",
            "12345",
        ],
        vec!["--synthetic-loopback-upstream", "127.0.0.1:1"],
        vec![
            "--synthetic-loopback-upstream",
            "127.0.0.1:1",
            "--port",
            "1",
        ],
        vec![
            "--synthetic-loopback-upstream",
            "127.0.0.1:1",
            "--port",
            "2",
            "--http-shutdown",
        ],
        vec![
            "--synthetic-loopback-upstream",
            "127.0.0.1:1",
            "--port",
            "2",
            "--dump-dir",
            "forbidden",
        ],
        vec![
            "--synthetic-loopback-upstream",
            "127.0.0.1:1",
            "--port",
            "2",
            "--server-info",
            "forbidden",
        ],
        vec![
            "--synthetic-loopback-upstream",
            "127.0.0.1:1",
            "--port",
            "2",
            "--upstream-url",
            "http://example.invalid",
        ],
    ] {
        let mut child = OwnedChild(
            binary()
                .unwrap()
                .args(args)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        assert!(!child.wait().unwrap().success());
    }
    let mut child = OwnedChild(binary().unwrap().stdin(Stdio::piped()).spawn().unwrap());
    sleep(Duration::from_millis(100));
    assert!(child.0.try_wait().unwrap().is_none());
    drop(child.0.stdin.take());
    assert!(!child.wait().unwrap().success());
}

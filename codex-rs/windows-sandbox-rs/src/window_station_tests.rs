//! #341: over SSH (a non-interactive window station), the elevated sandbox's
//! runner died with `0xC0000142` before connecting to Core, and the commands
//! it started died the same way. Each test reruns itself alone in a child
//! process that moves to a fresh window station of its own, with an SSH
//! session's name and DACLs.

// The probes' output is the evidence of these measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::DESKTOP_ACCESS;
use super::WINDOW_STATION_ACCESS;
use super::WindowAccessGrant;
use super::current_desktop_name;
use super::current_window_station_name;
use super::grant_window_access;
use super::object_grants;
use crate::desktop::LaunchDesktop;
use crate::logon_launch::LogonLaunchRequest;
use crate::logon_launch::create_process_with_logon;
use crate::winutil::resolve_sid;
use crate::winutil::string_from_sid_bytes;
use crate::winutil::to_wide;
use pretty_assertions::assert_eq;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::path::PathBuf;
use std::process::Command;
use std::ptr;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::HLOCAL;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::NetworkManagement::NetManagement::NERR_Success;
use windows_sys::Win32::NetworkManagement::NetManagement::NetUserAdd;
use windows_sys::Win32::NetworkManagement::NetManagement::NetUserDel;
use windows_sys::Win32::NetworkManagement::NetManagement::UF_DONT_EXPIRE_PASSWD;
use windows_sys::Win32::NetworkManagement::NetManagement::UF_SCRIPT;
use windows_sys::Win32::NetworkManagement::NetManagement::USER_INFO_1;
use windows_sys::Win32::NetworkManagement::NetManagement::USER_PRIV_USER;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::StationsAndDesktops::CloseDesktop;
use windows_sys::Win32::System::StationsAndDesktops::CloseWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::CreateDesktopW;
use windows_sys::Win32::System::StationsAndDesktops::CreateWindowStationW;
use windows_sys::Win32::System::StationsAndDesktops::OpenDesktopW;
use windows_sys::Win32::System::StationsAndDesktops::OpenWindowStationW;
use windows_sys::Win32::System::StationsAndDesktops::SetProcessWindowStation;
use windows_sys::Win32::System::StationsAndDesktops::SetThreadDesktop;
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
use windows_sys::Win32::System::Threading::CreateProcessW;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_SEC_WIN_341_ROLE";
const ON_FRESH_STATION: &str = "sec-win-341: on a fresh window station";
const NO_FRESH_STATION: &str =
    "sec-win-341: a normal session cannot create a window station here; skipped";
const STATUS_DLL_INIT_FAILED: u32 = 0xC000_0142;
const READ_CONTROL_AND_WRITE_DAC: u32 = 0x0002_0000 | 0x0004_0000;
/// The specific rights an SSH session gives its own user, and `READ_CONTROL`;
/// no `WRITE_DAC`.
const LIMITED_STATION_ACCESS: u32 = 0x006E | 0x0002_0000;
const LIMITED_DESKTOP_ACCESS: u32 = 0x00CF | 0x0002_0000;
/// Set where the fresh window station must be created (an elevated run), so
/// a skip fails instead of passing.
const REQUIRE_ENV: &str = "CODEX_SEC_WIN_341_REQUIRE";
const CWF_CREATE_ONLY: u32 = 0x0001;

/// In the parent: reruns `test` alone in a child that has moved to a fresh
/// window station, and checks it passed there. In that child: returns true,
/// and the test body runs. A normal (medium-integrity) session in session 0,
/// as over SSH, may not create window stations: then the test is skipped.
fn on_fresh_window_station(test: &str) -> bool {
    if std::env::var_os(ROLE_ENV).is_some() {
        if !enter_fresh_window_station() {
            eprintln!("{NO_FRESH_STATION}");
            return false;
        }
        eprintln!("{ON_FRESH_STATION}");
        return true;
    }
    let output = Command::new(std::env::current_exe().expect("test binary"))
        .args([test, "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE_ENV, "1")
        .output()
        .expect("rerun the test alone");
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprint!("{stderr}{}", String::from_utf8_lossy(&output.stdout));
    assert!(output.status.success(), "{}", output.status);
    // A rename would make the rerun match nothing and pass.
    if std::env::var_os(REQUIRE_ENV).is_some() {
        assert!(stderr.contains(ON_FRESH_STATION), "the rerun was skipped");
    } else {
        assert!(
            stderr.contains(ON_FRESH_STATION) || stderr.contains(NO_FRESH_STATION),
            "the rerun did not run"
        );
    }
    false
}

/// Moves this process, and this thread, to a new window station with a
/// `Default` desktop whose DACLs are those of a Windows OpenSSH session's
/// (Windows 11 26200): this user, and Administrators for a few rights.
/// Returns false when this session may not create one and is not elevated.
fn enter_fresh_window_station() -> bool {
    let user = current_user_sid();
    let user = string_from_sid_bytes(&user).expect("user SID string");
    let station_sddl = format!(
        "D:(A;;DCLCSWWPDTSDRCWDWO;;;{user})(A;OINPIO;CCDCLCSWDTLOSDRCWDWO;;;{user})(A;;CR;;;BA)(A;OINPIO;CCDTLO;;;BA)"
    );
    let desktop_sddl = format!("D:(A;;CCDCLCSWDTLOSDRCWDWO;;;{user})(A;;CCDTLO;;;BA)");
    let suffix = SmallRng::from_entropy().r#gen::<u32>();
    let station = to_wide(format!("Service-0x0-{suffix:x}$"));
    // Below, each object is created with the access its DACL grants this user
    // (at medium integrity, nothing more is granted).
    let station_attributes = SecurityAttributes::from_sddl(&station_sddl);
    // SAFETY: creates a window station this process keeps until it exits.
    let handle = unsafe {
        CreateWindowStationW(
            station.as_ptr(),
            CWF_CREATE_ONLY,
            0x000F_006E,
            &station_attributes.attributes,
        )
    };
    if handle == 0 {
        let err = std::io::Error::last_os_error();
        let elevated = crate::setup::is_elevated().expect("elevation");
        assert!(
            err.kind() == std::io::ErrorKind::PermissionDenied && !elevated,
            "create a window station: {err}"
        );
        return false;
    }
    // Like an SSH session's processes, this one holds its window station and
    // desktop without `WRITE_DAC`.
    // SAFETY: opens the window station created above; kept until exit.
    let limited = unsafe { OpenWindowStationW(station.as_ptr(), 0, LIMITED_STATION_ACCESS) };
    assert_ne!(limited, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: `limited` is the window station opened above.
    assert_ne!(unsafe { SetProcessWindowStation(limited) }, 0);
    // SAFETY: no longer needed; the process holds `limited`.
    unsafe { CloseWindowStation(handle) };
    let name = to_wide("Default");
    let desktop_attributes = SecurityAttributes::from_sddl(&desktop_sddl);
    // SAFETY: creates a desktop in the window station set above.
    let desktop = unsafe {
        CreateDesktopW(
            name.as_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            0x000F_00CF,
            &desktop_attributes.attributes,
        )
    };
    assert_ne!(desktop, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: opens the desktop created above; kept until exit.
    let limited = unsafe { OpenDesktopW(name.as_ptr(), 0, 0, LIMITED_DESKTOP_ACCESS) };
    assert_ne!(limited, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: this test thread has no windows or hooks yet.
    assert_ne!(
        unsafe { SetThreadDesktop(limited) },
        0,
        "{}",
        std::io::Error::last_os_error()
    );
    true
}

fn current_user_sid() -> Vec<u8> {
    // SAFETY: the token is closed right after reading its user.
    unsafe {
        let token = crate::token::get_current_token_for_restriction().expect("process token");
        let sid = crate::token::get_user_sid_bytes(token).expect("token user");
        CloseHandle(token);
        sid
    }
}

struct SecurityAttributes {
    attributes: SECURITY_ATTRIBUTES,
}

impl SecurityAttributes {
    fn from_sddl(sddl: &str) -> Self {
        let sddl = to_wide(sddl);
        let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
        // SAFETY: parses `sddl` into a descriptor freed on drop.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        };
        assert_ne!(ok, 0, "{}", std::io::Error::last_os_error());
        Self {
            attributes: SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: 0,
            },
        }
    }
}

impl Drop for SecurityAttributes {
    fn drop(&mut self) {
        // SAFETY: allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW.
        unsafe { LocalFree(self.attributes.lpSecurityDescriptor as HLOCAL) };
    }
}

fn system32(program: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"))
        .join("System32")
        .join(program)
}

/// Waits for `process` and returns its exit code, ending it after a while.
fn wait_exit_code(process: HANDLE) -> u32 {
    let mut code = 0u32;
    // SAFETY: `process` is a live process handle the caller owns.
    unsafe {
        if WaitForSingleObject(process, 30_000) != 0 {
            TerminateProcess(process, 1);
        }
        GetExitCodeProcess(process, &mut code);
        CloseHandle(process);
    }
    code
}

/// Starts `program` as this user on `desktop` and returns its exit code.
fn run_on_desktop(program: &PathBuf, desktop: *mut u16) -> u32 {
    let application = to_wide(program);
    let mut command_line = to_wide(program);
    // SAFETY: zeroed POD with its size set, as the API requires.
    let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    startup.lpDesktop = desktop;
    // SAFETY: zeroed POD filled in by the call.
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: every pointer refers to a live buffer above.
    let ok = unsafe {
        CreateProcessW(
            application.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            CREATE_NO_WINDOW,
            ptr::null(),
            ptr::null(),
            &startup,
            &mut info,
        )
    };
    assert_ne!(ok, 0, "{}", std::io::Error::last_os_error());
    // SAFETY: returned by the call above.
    unsafe { CloseHandle(info.hThread) };
    wait_exit_code(info.hProcess)
}

/// The commands the runner starts go on a desktop in the runner's own window
/// station. Before, they were sent to `Winsta0`, where the private desktop
/// (created in the runner's window station) does not exist.
#[test]
fn sec_win_341_commands_start_in_the_launchers_window_station() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_341_commands_start_in_the_launchers_window_station",
    ) {
        return;
    }
    let station = current_window_station_name().expect("window station");
    assert!(station.starts_with("Service-0x0-"), "{station}");
    let shared = LaunchDesktop::prepare(/*use_private_desktop*/ false, None).expect("desktop");
    let private = LaunchDesktop::prepare(/*use_private_desktop*/ true, None).expect("desktop");
    let name = |desktop: &LaunchDesktop| {
        let start = desktop.startup_info_desktop();
        let len = (0..).take_while(|&i| unsafe { *start.add(i) } != 0).count();
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(start, len) })
    };
    eprintln!(
        "sec-win-341: launch desktops {} and {}",
        name(&shared),
        name(&private)
    );
    // `whoami.exe` loads user32, so it fails to start on a desktop it can't
    // reach.
    let codes: Vec<u32> = [&shared, &private]
        .iter()
        .map(|desktop| run_on_desktop(&system32("whoami.exe"), desktop.startup_info_desktop()))
        .collect();
    eprintln!("sec-win-341: whoami exit codes {codes:x?}");
    assert_eq!(codes, vec![0, 0]);
    assert_eq!(name(&shared), format!("{station}\\Default"));
    assert!(
        name(&private).starts_with(&format!("{station}\\CodexSandboxDesktop-")),
        "{}",
        name(&private)
    );
}

/// A local user created for the test, deleted on drop.
struct TempUser {
    name: String,
    password: String,
}

impl TempUser {
    fn create() -> Self {
        let mut rng = SmallRng::from_entropy();
        let name = format!("CdxSec341{:06x}", rng.r#gen::<u32>() & 0xFF_FFFF);
        let password = format!("Aa1!{:032x}", rng.r#gen::<u128>());
        let name_w = to_wide(&name);
        let password_w = to_wide(&password);
        let info = USER_INFO_1 {
            usri1_name: name_w.as_ptr() as *mut u16,
            usri1_password: password_w.as_ptr() as *mut u16,
            usri1_password_age: 0,
            usri1_priv: USER_PRIV_USER,
            usri1_home_dir: ptr::null_mut(),
            usri1_comment: ptr::null_mut(),
            usri1_flags: UF_SCRIPT | UF_DONT_EXPIRE_PASSWD,
            usri1_script_path: ptr::null_mut(),
        };
        // SAFETY: `info` points at live buffers for the call.
        let status =
            unsafe { NetUserAdd(ptr::null(), 1, ptr::from_ref(&info).cast(), ptr::null_mut()) };
        assert_eq!(status, NERR_Success, "NetUserAdd {name}");
        Self { name, password }
    }
}

impl Drop for TempUser {
    fn drop(&mut self) {
        let name = to_wide(&self.name);
        // SAFETY: deletes the user this value created.
        let status = unsafe { NetUserDel(ptr::null(), name.as_ptr()) };
        eprintln!("sec-win-341: deleted {} ({status})", self.name);
        // A leaked account must show; not while a failure unwinds.
        if status != NERR_Success && !std::thread::panicking() {
            panic!("NetUserDel {} failed: {status}", self.name);
        }
    }
}

fn station_and_desktop_grant(sid: &[u8]) -> (bool, bool) {
    let station = to_wide(current_window_station_name().expect("window station"));
    let desktop = to_wide(current_desktop_name().expect("desktop"));
    // SAFETY: opened for reading their DACLs, closed below.
    unsafe {
        let station = OpenWindowStationW(station.as_ptr(), 0, READ_CONTROL_AND_WRITE_DAC);
        let desktop = OpenDesktopW(desktop.as_ptr(), 0, 0, READ_CONTROL_AND_WRITE_DAC);
        assert!(station != 0 && desktop != 0, "open window objects");
        let granted = (
            object_grants(station, sid, WINDOW_STATION_ACCESS).expect("station DACL"),
            object_grants(desktop, sid, DESKTOP_ACCESS).expect("desktop DACL"),
        );
        CloseWindowStation(station);
        CloseDesktop(desktop);
        granted
    }
}

/// Core grants the sandbox's user access to a non-interactive window station
/// and desktop before starting the runner as that user, once however often it
/// starts one. Elevated (it creates a local user), a process started as
/// another user from such a window station then starts; it used to die with
/// `0xC0000142`.
#[test]
fn sec_win_341_runner_user_can_start_on_a_non_interactive_window_station() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_341_runner_user_can_start_on_a_non_interactive_window_station",
    ) {
        return;
    }
    // Elevated, a process started as another user from here starts. Without
    // the grant it died with `0xC0000142`.
    if crate::setup::is_elevated().expect("elevation") {
        let user = TempUser::create();
        let whoami = system32("whoami.exe");
        let command_line = format!("\"{}\"", whoami.display());
        let launched = create_process_with_logon(
            &LogonLaunchRequest {
                username: &user.name,
                password: &user.password,
                application: &whoami,
                command_line: &command_line,
                cwd: &system32(""),
            },
            &whoami,
        )
        .expect("start whoami as the test user");
        // `whoami` stands in for the launcher, which must not be used here.
        assert!(!launched.via_launcher, "started through the launcher");
        assert_eq!(launched.window_access_error, None);
        let code = wait_exit_code(launched.process);
        eprintln!("sec-win-341: whoami as {} exited {code:#x}", user.name);
        assert_ne!(
            code, STATUS_DLL_INIT_FAILED,
            "could not attach to the window station"
        );
        assert_eq!(code, 0);
        let sid = resolve_sid(&user.name).expect("test user SID");
        assert_eq!(station_and_desktop_grant(&sid), (true, true));
    } else {
        eprintln!("sec-win-341: not elevated, so no local user for the logon launch; skipped");
    }

    // Any SID the fresh window station does not grant.
    let users = resolve_sid("Users").expect("Users SID");
    assert_eq!(station_and_desktop_grant(&users), (false, false));
    assert_eq!(
        grant_window_access("Users").expect("grant"),
        WindowAccessGrant::Granted
    );
    assert_eq!(station_and_desktop_grant(&users), (true, true));
    assert_eq!(
        grant_window_access("Users").expect("grant again"),
        WindowAccessGrant::AlreadyGranted
    );
}

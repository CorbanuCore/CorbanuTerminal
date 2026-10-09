//! #341: over SSH (a non-interactive window station), the elevated sandbox's
//! runner died with `0xC0000142` before connecting to Core, and the commands
//! it started died the same way. #345: the access Core gives them there is
//! for the runner's logon, while the runner runs, and its commands keep only
//! what they need to start. Each test reruns itself alone in a child process
//! on a non-interactive window station: this SSH session's own, or a fresh
//! one with an SSH session's name and DACLs.

// The probes' output is the evidence of these measured runs (`--nocapture`).
#![allow(clippy::print_stderr)]

use super::COMMAND_STATION_ACCESS;
use super::DESKTOP_ACCESS;
use super::RUNNER_STATION_ACCESS;
use super::WindowAccess;
use super::current_desktop_name;
use super::current_window_station_name;
use super::object_dacl;
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
use std::ffi::c_void;
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
use windows_sys::Win32::Security::ACE_HEADER;
use windows_sys::Win32::Security::ACL_SIZE_INFORMATION;
use windows_sys::Win32::Security::AclSizeInformation;
use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;
use windows_sys::Win32::Security::EqualSid;
use windows_sys::Win32::Security::GetAce;
use windows_sys::Win32::Security::GetAclInformation;
use windows_sys::Win32::Security::GetLengthSid;
use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Storage::FileSystem::READ_CONTROL;
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
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::Threading::GetExitCodeProcess;
use windows_sys::Win32::System::Threading::OpenProcessToken;
use windows_sys::Win32::System::Threading::PROCESS_INFORMATION;
use windows_sys::Win32::System::Threading::STARTUPINFOW;
use windows_sys::Win32::System::Threading::TerminateProcess;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

const ROLE_ENV: &str = "CODEX_SEC_WIN_341_ROLE";
const ON_FRESH_STATION: &str = "sec-win-341: on a fresh window station";
const ON_OWN_STATION: &str = "sec-win-341: on this session's non-interactive window station";
const NO_FRESH_STATION: &str =
    "sec-win-341: a normal session cannot create a window station here; skipped";
const STATUS_DLL_INIT_FAILED: u32 = 0xC000_0142;
/// The specific rights an SSH session gives its own user, and `READ_CONTROL`;
/// no `WRITE_DAC`.
const LIMITED_STATION_ACCESS: u32 = 0x006E | 0x0002_0000;
const LIMITED_DESKTOP_ACCESS: u32 = 0x00CF | 0x0002_0000;
/// Set where the fresh window station must be created (an elevated run), so
/// a skip fails instead of passing.
const REQUIRE_ENV: &str = "CODEX_SEC_WIN_341_REQUIRE";
const CWF_CREATE_ONLY: u32 = 0x0001;

/// In the parent: reruns `test` alone in a child on a non-interactive window
/// station, and checks it passed there. In that child: returns true, and the
/// test body runs. Over SSH (or in a service) that is this session's own
/// window station; elsewhere the child moves to a fresh one with an SSH
/// session's name and DACLs. A normal (medium-integrity) session may not
/// create window stations: then, off SSH, the test is skipped.
fn on_fresh_window_station(test: &str) -> bool {
    if std::env::var_os(ROLE_ENV).is_some() {
        let station = current_window_station_name().expect("window station");
        if !super::is_interactive_window_station(&station) {
            eprintln!("{ON_OWN_STATION} ({station})");
            return true;
        }
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
    let ran = stderr.contains(ON_FRESH_STATION) || stderr.contains(ON_OWN_STATION);
    // A rename would make the rerun match nothing and pass.
    if std::env::var_os(REQUIRE_ENV).is_some() {
        assert!(ran, "the rerun was skipped");
    } else {
        assert!(
            ran || stderr.contains(NO_FRESH_STATION),
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

/// The masks of the allow entries for `sid` on this process's window station
/// and on its desktop, in DACL order.
fn allow_entries(sid: &[u8]) -> (Vec<u32>, Vec<u32>) {
    let station = to_wide(current_window_station_name().expect("window station"));
    let desktop = to_wide(current_desktop_name().expect("desktop"));
    // SAFETY: opened for reading their DACLs, closed below.
    unsafe {
        let station = OpenWindowStationW(station.as_ptr(), 0, READ_CONTROL);
        let desktop = OpenDesktopW(desktop.as_ptr(), 0, 0, READ_CONTROL);
        assert!(station != 0 && desktop != 0, "open window objects");
        let entries = (
            object_allow_entries(station, sid),
            object_allow_entries(desktop, sid),
        );
        CloseWindowStation(station);
        CloseDesktop(desktop);
        entries
    }
}

fn object_allow_entries(handle: isize, sid: &[u8]) -> Vec<u32> {
    let (dacl, descriptor) = object_dacl(handle).expect("DACL");
    let mut masks = Vec::new();
    // SAFETY: reads `dacl`, owned by `descriptor` until it is freed below.
    unsafe {
        let mut info: ACL_SIZE_INFORMATION = std::mem::zeroed();
        if !dacl.is_null()
            && GetAclInformation(
                dacl,
                ptr::from_mut(&mut info).cast(),
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            ) != 0
        {
            for index in 0..info.AceCount {
                let mut ace: *mut c_void = ptr::null_mut();
                if GetAce(dacl, index, &mut ace) == 0 {
                    continue;
                }
                let header = &*(ace as *const ACE_HEADER);
                let mask = *((ace as usize + std::mem::size_of::<ACE_HEADER>()) as *const u32);
                let ace_sid = (ace as usize + std::mem::size_of::<ACE_HEADER>() + 4) as *mut c_void;
                if header.AceType == 0 && EqualSid(ace_sid, sid.as_ptr() as *mut c_void) != 0 {
                    masks.push(mask);
                }
            }
        }
        LocalFree(descriptor as HLOCAL);
    }
    masks
}

fn process_logon_sid_for_test(process: HANDLE) -> Vec<u8> {
    let mut token: HANDLE = 0;
    // SAFETY: `process` is live; the token is closed below.
    unsafe {
        assert_ne!(
            OpenProcessToken(process, TOKEN_QUERY, &mut token),
            0,
            "{}",
            std::io::Error::last_os_error()
        );
        let sid = crate::token::get_logon_sid_bytes(token).expect("logon SID");
        CloseHandle(token);
        sid
    }
}

/// A local user to start processes as: a new one when elevated, else the
/// one CI's normal-session step makes (`CODEX_PF27S06_LOGON_USER`, with its
/// password in `CODEX_PF27S06_LOGON_PASSWORD_FILE`).
enum LogonUser {
    Temp(TempUser),
    Given { name: String, password: String },
}

impl LogonUser {
    fn get() -> Option<Self> {
        if crate::setup::is_elevated().expect("elevation") {
            return Some(Self::Temp(TempUser::create()));
        }
        let name = std::env::var("CODEX_PF27S06_LOGON_USER").ok()?;
        let file = std::env::var_os("CODEX_PF27S06_LOGON_PASSWORD_FILE")?;
        let password = std::fs::read_to_string(file).expect("read the logon password");
        Some(Self::Given {
            name,
            password: password.trim_end().to_string(),
        })
    }

    fn name(&self) -> &str {
        match self {
            Self::Temp(user) => &user.name,
            Self::Given { name, .. } => name,
        }
    }

    fn password(&self) -> &str {
        match self {
            Self::Temp(user) => &user.password,
            Self::Given { password, .. } => password,
        }
    }
}

/// Core grants a runner's logon access to a non-interactive window station
/// and desktop before starting the runner, once however often it starts
/// one. Elevated (it creates a local user), a process started as another
/// user from such a window station then starts; it used to die with
/// `0xC0000142`.
#[test]
fn sec_win_341_runner_user_can_start_on_a_non_interactive_window_station() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_341_runner_user_can_start_on_a_non_interactive_window_station",
    ) {
        return;
    }
    let Some(user) = LogonUser::get() else {
        eprintln!("sec-win-341: no local user for the logon launch; skipped");
        return;
    };
    let whoami = system32("whoami.exe");
    let command_line = format!("\"{}\"", whoami.display());
    let launched = create_process_with_logon(
        &LogonLaunchRequest {
            username: user.name(),
            password: user.password(),
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
    eprintln!("sec-win-341: whoami as {} exited {code:#x}", user.name());
    assert_ne!(
        code, STATUS_DLL_INIT_FAILED,
        "could not attach to the window station"
    );
    assert_eq!(code, 0);
}

/// The mask #343 granted on the window station: what a runner needs.
const RUNNER_STATION_MASK: u32 = 0x0002_006E;
/// The same without `WINSTA_CREATEDESKTOP`: what its commands need.
const COMMAND_STATION_MASK: u32 = 0x0002_0066;
/// `DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS`.
const DESKTOP_MASK: u32 = 0x0081;

/// #345: the access is for the started process's logon SID, not its user
/// (whom every logon of that user on the machine has), and goes when Core
/// is done with the process. A command started after that point still
/// starts while it runs. On `main` the entries were for the user and stayed
/// until the window station went away.
#[test]
fn sec_win_345_access_is_for_the_runners_logon_and_its_lifetime() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_345_access_is_for_the_runners_logon_and_its_lifetime",
    ) {
        return;
    }
    let Some(user) = LogonUser::get() else {
        eprintln!("sec-win-345: no local user for the logon launch; skipped");
        return;
    };
    let user_sid = resolve_sid(user.name()).expect("user SID");
    let cmd = system32("cmd.exe");
    // `whoami` loads user32: the second one starts after the checks below.
    let command_line = format!(
        "\"{}\" /d /c \"whoami >nul && ping -n 4 127.0.0.1 >nul && whoami >nul\"",
        cmd.display()
    );
    let launched = create_process_with_logon(
        &LogonLaunchRequest {
            username: user.name(),
            password: user.password(),
            application: &cmd,
            command_line: &command_line,
            cwd: &system32(""),
        },
        &cmd,
    )
    .expect("start cmd as the test user");
    assert!(!launched.via_launcher, "started through the launcher");
    let logon_sid = process_logon_sid_for_test(launched.process);
    eprintln!(
        "sec-win-345: {} runs as logon {}",
        user.name(),
        string_from_sid_bytes(&logon_sid).expect("SID string")
    );
    let while_running = (allow_entries(&logon_sid), allow_entries(&user_sid));
    eprintln!("sec-win-345: entries while it runs (logon, user): {while_running:x?}");
    // Closes `launched.process`.
    let code = wait_exit_code(launched.process);
    eprintln!("sec-win-345: cmd exited {code:#x}");
    // Core is done with it: what it holds goes.
    drop(launched);
    let after = (allow_entries(&logon_sid), allow_entries(&user_sid));
    eprintln!("sec-win-345: entries after it exited (logon, user): {after:x?}");
    assert_eq!(code, 0, "a command started while it ran did not start");
    assert_eq!(
        while_running,
        (
            (vec![RUNNER_STATION_MASK], vec![DESKTOP_MASK]),
            (vec![], vec![])
        )
    );
    assert_eq!(after, ((vec![], vec![]), (vec![], vec![])));
}

/// Entries of one value are counted apart from another's for the same
/// logon (the runners of one session share it), narrowed in place, and
/// removed exactly.
#[test]
fn sec_win_345_window_access_counts_narrows_and_removes_exactly() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_345_window_access_counts_narrows_and_removes_exactly",
    ) {
        return;
    }
    // Any SID the window station does not grant.
    let users = resolve_sid("Users").expect("Users SID");
    assert_eq!(allow_entries(&users), (vec![], vec![]));
    let mut first = WindowAccess::grant_runner(&users)
        .expect("grant")
        .expect("not interactive");
    let second = WindowAccess::grant_runner(&users)
        .expect("grant again")
        .expect("not interactive");
    assert_eq!(first.sid(), users.as_slice());
    assert_eq!(
        allow_entries(&users),
        (
            vec![RUNNER_STATION_ACCESS, RUNNER_STATION_ACCESS],
            vec![DESKTOP_ACCESS, DESKTOP_ACCESS]
        )
    );
    first
        .narrow_for_commands(/*commands_use_this_desktop*/ false)
        .expect("narrow");
    assert_eq!(
        allow_entries(&users),
        (
            vec![RUNNER_STATION_ACCESS, COMMAND_STATION_ACCESS],
            vec![DESKTOP_ACCESS]
        )
    );
    drop(second);
    assert_eq!(
        allow_entries(&users),
        (vec![COMMAND_STATION_ACCESS], vec![])
    );
    let shared = WindowAccess::grant_desktop(&users)
        .expect("grant the desktop")
        .expect("not interactive");
    assert_eq!(
        allow_entries(&users),
        (vec![COMMAND_STATION_ACCESS], vec![DESKTOP_ACCESS])
    );
    drop(first);
    drop(shared);
    assert_eq!(allow_entries(&users), (vec![], vec![]));
    assert_eq!(
        (
            RUNNER_STATION_ACCESS,
            COMMAND_STATION_ACCESS,
            DESKTOP_ACCESS
        ),
        (RUNNER_STATION_MASK, COMMAND_STATION_MASK, DESKTOP_MASK)
    );
}

const PROBE_ENV: &str = "CODEX_SEC_WIN_345_PROBE";
const PROBE_TEST: &str = "window_station::tests::window_station_probe_in_the_sandbox";
const PROBE_LINE: &str = "sec-win-345 probe:";

/// Not a test of its own: the command [`sec_win_345_sandboxed_commands_dont_get_the_runners_access`]
/// runs in the elevated sandbox. Reports whether it can create a desktop on
/// its window station and open Core's desktop, once Core has had a moment to
/// narrow its access, and whether a child it starts then starts.
#[test]
fn window_station_probe_in_the_sandbox() {
    let Some(core_desktop) = std::env::var_os(PROBE_ENV) else {
        return;
    };
    let station = to_wide(current_window_station_name().expect("window station"));
    let core_desktop = to_wide(core_desktop);
    let probe = || {
        // SAFETY: each handle is closed right after the check.
        unsafe {
            let station = OpenWindowStationW(station.as_ptr(), 0, 0x0008);
            let desktop = OpenDesktopW(core_desktop.as_ptr(), 0, 0, 0x0001);
            if station != 0 {
                CloseWindowStation(station);
            }
            if desktop != 0 {
                CloseDesktop(desktop);
            }
            (station != 0, desktop != 0)
        }
    };
    // Core narrows the access right after the runner reports the command
    // started; this command may get here first.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut access = probe();
    while access.0 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
        access = probe();
    }
    let child = Command::new(system32("whoami.exe"))
        .output()
        .map(|output| output.status.code().unwrap_or(-1) as u32);
    // SAFETY: the pseudo-handle of this process.
    let logon = process_logon_sid_for_test(unsafe { GetCurrentProcess() });
    let logon = string_from_sid_bytes(&logon).expect("SID");
    let user = string_from_sid_bytes(&current_user_sid()).expect("SID");
    println!(
        "{PROBE_LINE} create_desktop={} core_desktop={} child={:x?} logon={logon} user={user}",
        access.0, access.1, child
    );
}

/// #345: the commands the elevated sandbox's runner starts don't keep the
/// runner's access: they can't create desktops on Core's window station, and
/// can't open Core's desktop unless they run on it (no private desktop). A
/// child they start still starts. When the runner exits, its logon's
/// entries go, and none were ever for the sandbox's user. On `main` the
/// commands held all of it through the sandbox user, until the SSH session
/// ended. Elevated: the sandbox's setup needs it.
#[test]
fn sec_win_345_sandboxed_commands_dont_get_the_runners_access() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_345_sandboxed_commands_dont_get_the_runners_access",
    ) {
        return;
    }
    if !crate::setup::is_elevated().expect("elevation") {
        eprintln!("sec-win-345: the sandbox's setup needs an elevated run; skipped");
        return;
    }
    let core_desktop = current_desktop_name().expect("desktop");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let codex_home = tempfile::tempdir().expect("codex home");
    let cwd = std::env::current_dir().expect("cwd");
    let mut results = Vec::new();
    for private_desktop in [true, false] {
        let env_map = std::collections::HashMap::from([
            (PROBE_ENV.to_string(), core_desktop.clone()),
            (
                "SystemRoot".to_string(),
                std::env::var("SystemRoot").expect("SystemRoot"),
            ),
        ]);
        let command = vec![
            std::env::current_exe()
                .expect("test binary")
                .to_string_lossy()
                .into_owned(),
            PROBE_TEST.to_string(),
            "--exact".to_string(),
            "--nocapture".to_string(),
            "--test-threads=1".to_string(),
        ];
        let workspace = codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(&cwd)
            .expect("absolute cwd");
        let stdout = runtime.block_on(async {
            let spawned = crate::spawn_windows_sandbox_session_elevated_for_permission_profile(
                &codex_protocol::models::PermissionProfile::read_only(),
                std::slice::from_ref(&workspace),
                codex_home.path(),
                command,
                &cwd,
                env_map,
                /*proxy_enforced*/ false,
                /*network_proxy_restricting_sid*/ None,
                Some(60_000),
                /*read_roots_override*/ None,
                /*read_roots_include_platform_defaults*/ true,
                /*write_roots_override*/ None,
                &crate::DenyReadTargets::default(),
                &[],
                /*tty*/ false,
                /*stdin_open*/ false,
                private_desktop,
            )
            .await
            .expect("start the probe in the elevated sandbox");
            let codex_utils_pty::SpawnedProcess {
                session: _session,
                mut stdout_rx,
                stderr_rx: _stderr_rx,
                exit_rx,
            } = spawned;
            let mut stdout = Vec::new();
            while let Some(chunk) = stdout_rx.recv().await {
                stdout.extend(chunk);
            }
            let code = exit_rx.await.unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&stdout).into_owned();
            assert_eq!(code, 0, "probe failed: {stdout}");
            stdout
        });
        let line = stdout
            .lines()
            .find(|line| line.starts_with(PROBE_LINE))
            .unwrap_or_else(|| panic!("no probe report in {stdout}"))
            .to_string();
        eprintln!("sec-win-345: private desktop {private_desktop}: {line}");
        let field = |name: &str| {
            line.split_whitespace()
                .find_map(|part| part.strip_prefix(&format!("{name}=")))
                .unwrap_or_else(|| panic!("no {name} in {line}"))
                .to_string()
        };
        let sid = |name: &str| {
            let text = to_wide(field(name));
            let mut psid: *mut c_void = ptr::null_mut();
            // SAFETY: parses a SID string into a buffer copied and freed below.
            unsafe {
                assert_ne!(ConvertStringSidToSidW(text.as_ptr(), &mut psid), 0);
                let len = GetLengthSid(psid) as usize;
                let bytes = std::slice::from_raw_parts(psid as *const u8, len).to_vec();
                LocalFree(psid as HLOCAL);
                bytes
            }
        };
        let (logon, user) = (sid("logon"), sid("user"));
        // The runner exits right after its command; Core then removes the
        // entries.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut after = (allow_entries(&logon), allow_entries(&user));
        while after != ((vec![], vec![]), (vec![], vec![])) && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(100));
            after = (allow_entries(&logon), allow_entries(&user));
        }
        eprintln!("sec-win-345: entries after the runner exited (logon, user): {after:x?}");
        results.push((
            field("create_desktop"),
            field("core_desktop"),
            field("child"),
            after,
        ));
    }
    let gone = ((vec![], vec![]), (vec![], vec![]));
    assert_eq!(
        results,
        vec![
            (
                "false".to_string(),
                "false".to_string(),
                "Ok(0)".to_string(),
                gone.clone()
            ),
            (
                "false".to_string(),
                "true".to_string(),
                "Ok(0)".to_string(),
                gone
            ),
        ]
    );
}

/// #345: with the unelevated sandbox and `sandbox_private_desktop = false`,
/// commands started on Core's own non-interactive desktop died with
/// `0xC0000142`: their restricted tokens' restricting SIDs don't include the
/// user the desktop grants. Core now gives its own logon SID (which they
/// have) access to its desktop while they run.
#[test]
fn sec_win_345_unelevated_commands_start_on_cores_own_desktop() {
    if !on_fresh_window_station(
        "window_station::tests::sec_win_345_unelevated_commands_start_on_cores_own_desktop",
    ) {
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let codex_home = tempfile::tempdir().expect("codex home");
    let cwd = std::env::current_dir().expect("cwd");
    let workspace =
        codex_utils_absolute_path::AbsolutePathBuf::from_absolute_path(&cwd).expect("absolute cwd");
    let (code, stdout) = runtime.block_on(async {
        let spawned = crate::spawn_windows_sandbox_session_legacy(
            &codex_protocol::models::PermissionProfile::read_only(),
            std::slice::from_ref(&workspace),
            codex_home.path(),
            vec![system32("whoami.exe").to_string_lossy().into_owned()],
            &cwd,
            std::collections::HashMap::new(),
            Some(30_000),
            &[],
            &[],
            /*tty*/ false,
            /*stdin_open*/ false,
            /*use_private_desktop*/ false,
        )
        .await
        .expect("start whoami in the unelevated sandbox");
        let codex_utils_pty::SpawnedProcess {
            session: _session,
            mut stdout_rx,
            stderr_rx: _stderr_rx,
            exit_rx,
        } = spawned;
        let mut stdout = Vec::new();
        while let Some(chunk) = stdout_rx.recv().await {
            stdout.extend(chunk);
        }
        (
            exit_rx.await.unwrap_or(-1),
            String::from_utf8_lossy(&stdout).into_owned(),
        )
    });
    eprintln!("sec-win-345: unelevated whoami on Core's desktop exited {code:#x}: {stdout}");
    assert_eq!(code, 0);
    assert!(stdout.contains('\\'), "{stdout}");
    // The entry goes once the command is done.
    // SAFETY: the pseudo-handle of this process.
    let logon = process_logon_sid_for_test(unsafe { GetCurrentProcess() });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while allow_entries(&logon).1 != Vec::<u32>::new() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert_eq!(allow_entries(&logon).1, Vec::<u32>::new());
}

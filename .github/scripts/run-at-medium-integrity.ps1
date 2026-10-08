# Runs a program with the token UAC gives an administrator's normal session:
# Administrators deny-only, no privileges, medium integrity. GitHub's Windows
# runners are elevated, so this is how CI covers a normal (non-elevated)
# session (PF-27-S06, #294/#295). The program inherits this environment;
# its output is printed and its exit code is returned.
param(
    [Parameter(Mandatory = $true)][string]$Program,
    [string[]]$Arguments = @()
)
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class MediumIntegrity {
    [StructLayout(LayoutKind.Sequential)] struct SidAndAttributes { public IntPtr Sid; public uint Attributes; }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct StartupInfo {
        public int cb; public string reserved, desktop, title;
        public int x, y, xSize, ySize, xChars, yChars, fill, flags;
        public short show, reserved2; public IntPtr reserved3, stdIn, stdOut, stdErr;
    }
    [StructLayout(LayoutKind.Sequential)] struct ProcessInfo { public IntPtr process, thread; public int pid, tid; }
    [StructLayout(LayoutKind.Sequential)] struct SecurityAttributes { public int length; public IntPtr descriptor; public bool inherit; }

    [DllImport("advapi32.dll", SetLastError = true)] static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool ConvertStringSidToSidW(string sid, out IntPtr psid);
    [DllImport("advapi32.dll", SetLastError = true)] static extern bool CreateRestrictedToken(IntPtr token, uint flags, uint disableCount, SidAndAttributes[] disable, uint deleteCount, IntPtr delete, uint restrictCount, IntPtr restrict, out IntPtr newToken);
    [DllImport("advapi32.dll", SetLastError = true)] static extern bool SetTokenInformation(IntPtr token, int infoClass, ref SidAndAttributes info, uint length);
    [DllImport("advapi32.dll")] static extern uint GetLengthSid(IntPtr sid);
    [DllImport("advapi32.dll", SetLastError = true)] static extern bool SetTokenInformation(IntPtr token, int infoClass, ref IntPtr info, uint length);
    [DllImport("advapi32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool ConvertStringSecurityDescriptorToSecurityDescriptorW(string sddl, uint revision, out IntPtr sd, IntPtr size);
    [DllImport("advapi32.dll", SetLastError = true)] static extern bool GetSecurityDescriptorDacl(IntPtr sd, out bool present, out IntPtr dacl, out bool defaulted);
    [DllImport("advapi32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool CreateProcessAsUserW(IntPtr token, string app, System.Text.StringBuilder cmd, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref StartupInfo si, out ProcessInfo pi);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern IntPtr CreateFileW(string name, uint access, uint share, ref SecurityAttributes sa, uint disposition, uint flags, IntPtr template);
    [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
    [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr handle, uint ms);
    [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr process, out uint code);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);

    static void Check(bool ok, string what) {
        if (!ok) throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error(), what);
    }

    public static uint Run(string commandLine, string logPath, string userSid) {
        IntPtr token, restricted, admins, medium, user, descriptor, dacl;
        bool present, defaulted;
        // TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT
        Check(OpenProcessToken(GetCurrentProcess(), 0x2 | 0x8 | 0x1 | 0x80, out token), "OpenProcessToken");
        Check(ConvertStringSidToSidW("S-1-5-32-544", out admins), "Administrators SID");
        var disable = new[] { new SidAndAttributes { Sid = admins, Attributes = 0 } };
        // DISABLE_MAX_PRIVILEGE
        Check(CreateRestrictedToken(token, 0x1, 1, disable, 0, IntPtr.Zero, 0, IntPtr.Zero, out restricted), "CreateRestrictedToken");
        Check(ConvertStringSidToSidW("S-1-16-8192", out medium), "medium integrity SID");
        // TokenIntegrityLevel with SE_GROUP_INTEGRITY
        var label = new SidAndAttributes { Sid = medium, Attributes = 0x20 };
        Check(SetTokenInformation(restricted, 25, ref label, (uint)Marshal.SizeOf(label) + GetLengthSid(medium)), "lower the integrity level");
        // An elevated token's owner and default DACL are Administrators, which is
        // now deny-only: the new process could not open its own objects. Use the
        // user instead, as UAC's filtered token does.
        Check(ConvertStringSidToSidW(userSid, out user), "user SID");
        Check(SetTokenInformation(restricted, 4, ref user, (uint)IntPtr.Size), "TokenOwner");
        Check(ConvertStringSecurityDescriptorToSecurityDescriptorW("D:(A;;GA;;;" + userSid + ")(A;;GA;;;SY)", 1, out descriptor, IntPtr.Zero), "default DACL");
        Check(GetSecurityDescriptorDacl(descriptor, out present, out dacl, out defaulted), "default DACL");
        Check(SetTokenInformation(restricted, 6, ref dacl, (uint)IntPtr.Size), "TokenDefaultDacl");
        var sa = new SecurityAttributes { length = Marshal.SizeOf(typeof(SecurityAttributes)), inherit = true };
        // GENERIC_WRITE, share read, CREATE_ALWAYS
        IntPtr log = CreateFileW(logPath, 0x40000000, 0x1, ref sa, 2, 0x80, IntPtr.Zero);
        Check(log != new IntPtr(-1), "create the log");
        var si = new StartupInfo { cb = Marshal.SizeOf(typeof(StartupInfo)), flags = 0x100, stdOut = log, stdErr = log };
        ProcessInfo pi;
        // CREATE_NO_WINDOW
        Check(CreateProcessAsUserW(restricted, null, new System.Text.StringBuilder(commandLine), IntPtr.Zero, IntPtr.Zero, true, 0x08000000, IntPtr.Zero, null, ref si, out pi), "CreateProcessAsUserW");
        CloseHandle(log);
        WaitForSingleObject(pi.process, 0xFFFFFFFF);
        uint code;
        GetExitCodeProcess(pi.process, out code);
        CloseHandle(pi.thread); CloseHandle(pi.process); CloseHandle(restricted); CloseHandle(token);
        return code;
    }
}
'@

function Quote([string]$arg) {
    if ($arg -notmatch '[\s"]') { return $arg }
    return '"' + ($arg -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
}
$commandLine = ((@($Program) + $Arguments) | ForEach-Object { Quote $_ }) -join ' '
$log = Join-Path ([IO.Path]::GetTempPath()) ("medium-integrity-{0}.log" -f [guid]::NewGuid())
$userSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$code = [MediumIntegrity]::Run($commandLine, $log, $userSid)
Get-Content -Raw $log | Write-Output
Remove-Item $log
if ($code -ne 0) { Write-Output ("medium-integrity run exited with 0x{0:X8}" -f $code) }
# Exit codes are signed; NTSTATUS values such as 0xC0000142 must stay nonzero.
exit [BitConverter]::ToInt32([BitConverter]::GetBytes([uint32]$code), 0)

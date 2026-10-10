# #391 (Windows): runs a program in a job object that allows one active
# process, so it cannot start any child process (such as the credential
# broker). Usage: jobrun391.ps1 <program> [args...]
param([Parameter(Mandatory = $true)][string]$Program, [Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Job391 {
  [StructLayout(LayoutKind.Sequential)] struct BASIC { public long PerProcessUserTimeLimit; public long PerJobUserTimeLimit; public uint LimitFlags; public UIntPtr MinimumWorkingSetSize; public UIntPtr MaximumWorkingSetSize; public uint ActiveProcessLimit; public UIntPtr Affinity; public uint PriorityClass; public uint SchedulingClass; }
  [StructLayout(LayoutKind.Sequential)] struct IO { public ulong a, b, c, d, e, f; }
  [StructLayout(LayoutKind.Sequential)] struct EXT { public BASIC Basic; public IO Io; public UIntPtr ProcessMemoryLimit; public UIntPtr JobMemoryLimit; public UIntPtr PeakProcessMemoryUsed; public UIntPtr PeakJobMemoryUsed; }
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct STARTUPINFO { public int cb; public string r, d, t; public int x, y, xs, ys, xc, yc, fill, flags; public short show, r2; public IntPtr r3, i, o, e; }
  [StructLayout(LayoutKind.Sequential)] struct PI { public IntPtr hProcess, hThread; public int pid, tid; }
  [DllImport("kernel32.dll", SetLastError = true)] static extern IntPtr CreateJobObjectW(IntPtr a, string n);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool SetInformationJobObject(IntPtr j, int c, ref EXT i, int l);
  [DllImport("kernel32.dll", SetLastError = true)] static extern bool AssignProcessToJobObject(IntPtr j, IntPtr p);
  [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool CreateProcessW(string app, System.Text.StringBuilder cmd, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref STARTUPINFO si, out PI pi);
  [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr t);
  [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h, uint ms);
  [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr h, out uint code);
  public static int Run(string cmdline) {
    IntPtr job = CreateJobObjectW(IntPtr.Zero, null);
    var ext = new EXT();
    ext.Basic.LimitFlags = 0x8; // JOB_OBJECT_LIMIT_ACTIVE_PROCESS
    ext.Basic.ActiveProcessLimit = 1;
    if (!SetInformationJobObject(job, 9, ref ext, Marshal.SizeOf(typeof(EXT)))) throw new Exception("job limit: " + Marshal.GetLastWin32Error());
    var si = new STARTUPINFO(); si.cb = Marshal.SizeOf(typeof(STARTUPINFO));
    PI pi;
    if (!CreateProcessW(null, new System.Text.StringBuilder(cmdline), IntPtr.Zero, IntPtr.Zero, true, 0x4 /* CREATE_SUSPENDED */, IntPtr.Zero, null, ref si, out pi)) throw new Exception("create: " + Marshal.GetLastWin32Error());
    if (!AssignProcessToJobObject(job, pi.hProcess)) throw new Exception("assign: " + Marshal.GetLastWin32Error());
    ResumeThread(pi.hThread);
    WaitForSingleObject(pi.hProcess, 0xFFFFFFFF);
    uint code; GetExitCodeProcess(pi.hProcess, out code);
    return (int)code;
  }
}
"@
$quote = { param($a) if ($a -match '[\s"]' -or $a -eq '') { '"' + ($a -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"' } else { $a } }
$line = (@($Program) + @($Rest) | ForEach-Object { & $quote $_ }) -join ' '
exit [Job391]::Run($line)

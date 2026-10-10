# squat390.ps1 -Log <file> -Marker <text> [-Instances 200] [-WaitSeconds 120]
# #390 demo squatter: a same-user process that waits for the pipes of the
# Corbanu credential broker started under a Corbanu whose command line holds
# -Marker (other brokers on the machine are left alone), tries to add an
# instance to the control pipe, and adds -Instances instances to the data pipe. Each connection it gets is logged with
# the client's process id, the bytes received and whether they held an HTTP
# request or a bearer key. It never sends anything. Stop it with Stop-Process.
param([string]$Log, [string]$Marker, [int]$Instances = 200, [int]$WaitSeconds = 120)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class Squat390 {
  [DllImport("kernel32.dll", SetLastError = true)]
  static extern bool GetNamedPipeClientProcessId(IntPtr pipe, out uint pid);
  [DllImport("kernel32.dll", SetLastError = true)]
  static extern bool GetNamedPipeServerProcessId(IntPtr pipe, out uint pid);
  // The process that created the pipe (the broker drops this client unread).
  public static uint ServerPid(string name) {
    try {
      using (var client = new NamedPipeClientStream(".", name, PipeDirection.In)) {
        client.Connect(2000);
        uint pid = 0;
        GetNamedPipeServerProcessId(client.SafePipeHandle.DangerousGetHandle(), out pid);
        return pid;
      }
    } catch (Exception) { return 0; }
  }
  public static void Start(string name, int count) {
    for (int i = 0; i < count; i++) {
      var thread = new Thread(Serve);
      thread.IsBackground = true;
      thread.Start(name);
    }
  }
  static readonly object Gate = new object();
  public static string LogPath;
  public static void Log(string line) {
    lock (Gate) { File.AppendAllText(LogPath, DateTime.Now.ToString("HH:mm:ss") + " " + line + Environment.NewLine); }
  }
  public static void Serve(object state) {
    string name = (string)state;
    while (true) {
      NamedPipeServerStream server;
      try {
        server = new NamedPipeServerStream(name, PipeDirection.InOut, NamedPipeServerStream.MaxAllowedServerInstances, PipeTransmissionMode.Byte, PipeOptions.Asynchronous);
      } catch (Exception error) { Log("data pipe: could not add an instance: " + error.Message); return; }
      using (server) {
        try { server.WaitForConnection(); } catch (IOException) { }
        uint pid = 0;
        GetNamedPipeClientProcessId(server.SafePipeHandle.DangerousGetHandle(), out pid);
        var received = new MemoryStream();
        var buffer = new byte[4096];
        while (true) {
          var read = server.ReadAsync(buffer, 0, buffer.Length);
          if (!read.Wait(3000) || read.Result <= 0) break;
          received.Write(buffer, 0, read.Result);
        }
        string text = Encoding.ASCII.GetString(received.ToArray());
        bool http = text.Contains("HTTP/1.1");
        bool bearer = text.IndexOf("bearer", StringComparison.OrdinalIgnoreCase) >= 0;
        Log(String.Format("data pipe: connection from pid {0}: received {1} bytes; HTTP request: {2}; key: {3}",
          pid, received.Length, http ? "YES" : "no", bearer ? "YES" : "no"));
      }
    }
  }
}
"@
[Squat390]::LogPath = $Log
Set-Content -Path $Log -Value "squatter pid $PID started"
$deadline = (Get-Date).AddSeconds($WaitSeconds)
function Test-Ancestry([uint32]$ProcessId) {
  for ($depth = 0; $depth -lt 5 -and $ProcessId -ne 0; $depth++) {
    $process = Get-CimInstance Win32_Process -Filter "ProcessId = $ProcessId"
    if (-not $process) { return $false }
    if ($process.CommandLine -and $process.CommandLine.Contains($Marker)) { return $true }
    $ProcessId = $process.ParentProcessId
  }
  return $false
}
$control = $null; $data = $null
do {
  $nonces = [IO.Directory]::GetFiles('\\.\pipe\') | ForEach-Object { Split-Path $_ -Leaf } |
    Where-Object { $_ -like 'corbanu-cbk-*-b' } | ForEach-Object { $_.Substring(0, $_.Length - 2) }
  foreach ($nonce in $nonces) {
    if (Test-Ancestry ([Squat390]::ServerPid("$nonce-b"))) { $control = "$nonce-c"; $data = "$nonce-b"; break }
  }
  if ($data) { break }
  Start-Sleep -Milliseconds 300
} while ((Get-Date) -lt $deadline)
if (-not $data) { [Squat390]::Log('no broker pipes found for this run'); exit 1 }
[Squat390]::Log("found broker pipes $control and $data")
try {
  $extra = New-Object IO.Pipes.NamedPipeServerStream($control, [IO.Pipes.PipeDirection]::InOut, -1)
  [Squat390]::Log('control pipe: ADDED an instance')
} catch {
  [Squat390]::Log("control pipe: could not add an instance: $($_.Exception.InnerException.Message)$($_.Exception.Message)")
}
[Squat390]::Start($data, $Instances)
[Squat390]::Log("data pipe: added $Instances instances")
while ($true) { Start-Sleep 1 }

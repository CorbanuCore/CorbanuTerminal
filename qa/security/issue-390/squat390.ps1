# squat390.ps1 -Log <file> -Since <time> [-Instances 200] [-WaitSeconds 120]
# #390 demo squatter: a same-user process that waits for the pipes of the one
# Corbanu credential broker started after -Since (the broker's parent process
# is not Core, so it is found by its start time; with several such brokers it
# gives up and touches none), tries to add an instance to the control pipe,
# and adds -Instances instances to the data pipe. Each connection it gets is logged with
# the client's process id, the bytes received and whether they held an HTTP
# request or a bearer key. It never sends anything. Stop it with Stop-Process.
param([string]$Log, [datetime]$Since, [int]$Instances = 200, [int]$WaitSeconds = 120)
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
$control = $null; $data = $null
do {
  $found = @()
  $nonces = [IO.Directory]::GetFiles('\\.\pipe\') | ForEach-Object { [IO.Path]::GetFileName($_) } |
    Where-Object { $_ -like 'corbanu-cbk-*-b' } | ForEach-Object { $_.Substring(0, $_.Length - 2) }
  foreach ($nonce in $nonces) {
    $server = [Squat390]::ServerPid("$nonce-b")
    $process = if ($server) { Get-CimInstance Win32_Process -Filter "ProcessId = $server" }
    if ($process -and $process.Name -like 'corbanu*' -and $process.CreationDate -ge $Since) { $found += $nonce }
  }
  if ($found.Count -gt 1) { [Squat390]::Log("several brokers started since $Since; touching none"); exit 1 }
  if ($found.Count -eq 1) { $control = "$($found[0])-c"; $data = "$($found[0])-b"; break }
  Start-Sleep -Milliseconds 300
} while ((Get-Date) -lt $deadline)
if (-not $data) { [Squat390]::Log('no broker started since then'); exit 1 }
[Squat390]::Log("found broker pipes $control and $data")
try {
  $extra = New-Object IO.Pipes.NamedPipeServerStream($control, [IO.Pipes.PipeDirection]::InOut, -1)
  [Squat390]::Log('control pipe: ADDED an instance')
} catch {
  $reason = if ($_.Exception.InnerException) { $_.Exception.InnerException.Message } else { $_.Exception.Message }
  [Squat390]::Log("control pipe: could not add an instance: $($reason.Trim())")
}
[Squat390]::Start($data, $Instances)
# Clients reach the longest-listening instance first (measured). One client
# visit makes the broker replace its listening instance, which then queues
# behind ours.
Start-Sleep -Seconds 1
[void][Squat390]::ServerPid($data)
[Squat390]::Log("data pipe: added $Instances instances")
while ($true) { Start-Sleep 1 }

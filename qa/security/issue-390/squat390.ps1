# squat390.ps1 -Log <file> -TuiLog <codex-tui.log> [-Instances 200] [-WaitSeconds 120] [-LifetimeSeconds 600]
# #390 demo squatter: a same-user process that squats the named pipes of one
# Corbanu credential broker. It reads that broker's data pipe from the given
# Corbanu log ("isolated credential broker started ... data_endpoint=..."), so
# no other broker on the machine is touched. It tries to add an instance to the
# control pipe and adds -Instances instances to the data pipe. Each connection
# it gets is logged with the client's process id, the bytes received, and
# whether they held an HTTP request or a signed broker frame. It never sends
# anything, and exits after -LifetimeSeconds.
param(
  [string]$Log,
  [string]$TuiLog,
  [int]$Instances = 200,
  [int]$WaitSeconds = 120,
  [int]$LifetimeSeconds = 600
)
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
  static readonly object Gate = new object();
  public static string LogPath;
  public static void Log(string line) {
    lock (Gate) { File.AppendAllText(LogPath, DateTime.Now.ToString("HH:mm:ss") + " " + line + Environment.NewLine); }
  }
  // Visits the pipe once as a client (the broker drops it unread).
  public static void Visit(string name) {
    try {
      using (var client = new NamedPipeClientStream(".", name, PipeDirection.In)) { client.Connect(2000); }
    } catch (Exception) { }
  }
  public static void Start(string name, int count) {
    for (int i = 0; i < count; i++) {
      var thread = new Thread(Serve);
      thread.IsBackground = true;
      thread.Start(name);
    }
  }
  static void Serve(object state) {
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
        bool frame = text.IndexOf("x-corbanu-broker-frame", StringComparison.OrdinalIgnoreCase) >= 0;
        Log(String.Format("data pipe: connection from pid {0}: received {1} bytes; HTTP request: {2}; frame: {3}",
          pid, received.Length, http ? "YES" : "no", frame ? "YES" : "no"));
      }
    }
  }
}
"@
[Squat390]::LogPath = $Log
Set-Content -Path $Log -Value "squatter pid $PID started"
$deadline = (Get-Date).AddSeconds($WaitSeconds)
$data = $null
do {
  if (Test-Path $TuiLog) {
    $match = Select-String -Path $TuiLog -Pattern 'isolated credential broker started.*data_endpoint=\\\\\.\\pipe\\(corbanu-cbk-[0-9a-f]+)-b' |
      Select-Object -Last 1
    if ($match) { $data = "$($match.Matches[0].Groups[1].Value)-b" }
  }
  if (-not $data) { Start-Sleep -Milliseconds 300 }
} while (-not $data -and (Get-Date) -lt $deadline)
if (-not $data) { [Squat390]::Log('the log names no broker'); exit 1 }
$control = $data.Substring(0, $data.Length - 2) + '-c'
[Squat390]::Log("found broker pipes $control and $data")
try {
  $extra = New-Object IO.Pipes.NamedPipeServerStream($control, [IO.Pipes.PipeDirection]::InOut, -1)
  [Squat390]::Log('control pipe: ADDED an instance')
} catch {
  $reason = if ($_.Exception.InnerException) { $_.Exception.InnerException.Message } else { $_.Exception.Message }
  [Squat390]::Log("control pipe: could not add an instance: $($reason.Trim())")
}
[Squat390]::Start($data, $Instances)
# Clients reach the longest-listening instance first (measured). One visit
# makes the broker replace its listening instance, which then queues behind
# ours.
Start-Sleep -Seconds 1
[Squat390]::Visit($data)
[Squat390]::Log("data pipe: added $Instances instances")
Start-Sleep -Seconds $LifetimeSeconds

$ErrorActionPreference = "Stop"
$root = "C:\CorbanuQA\closeout-20261022"
$expected = "6919A1B6BC3DA7C5B964AA3BFF68F186CB0B1E3CF7C0106FAF9BF5588AD89FFF"
New-Item -ItemType Directory -Force -Path "$root\bin" | Out-Null
Copy-Item "$root\target\dev-small\corbanu.exe" "$root\bin\corbanu.exe"
if ((Get-FileHash "$root\bin\corbanu.exe" -Algorithm SHA256).Hash -ne $expected) { throw "Binary copy hash mismatch" }
$dirty = & git -C "$root\src" status --porcelain
if ($LASTEXITCODE -ne 0 -or $dirty) { throw "Source worktree is not clean; preserved" }
& git -C C:\CorbanuQA\src worktree remove "$root\src"
if ($LASTEXITCODE -ne 0) { throw "Worktree removal failed; target preserved" }
Remove-Item -LiteralPath "$root\target" -Recurse -Force
$fixtures = @(Get-ChildItem $root -Directory -Filter "fixture-*")
if ($fixtures.Count -ne 0) { throw "Unexpected fixture directory remains; inspect without reading secrets" }
Write-Output "CLEANUP_OK: candidate binary and logs retained; owned target/source removed; no fixture remains."

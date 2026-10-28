param([string] $ExecutablePath)

$ErrorActionPreference = 'Stop'
if ($env:LOCALAPPDATA -match '(?i)\\Packages\\sandbox\.\{[^}]+\}\\AC(?:\\|$)') {
    throw 'Run this launcher outside the restricted sandbox so Texpio inherits the normal Windows profile environment.'
}
$repository = (Resolve-Path -LiteralPath (Split-Path -Parent $PSScriptRoot)).Path
$expected = (Resolve-Path -LiteralPath (Join-Path $repository 'src-tauri\target\release\texpio.exe')).Path
if ($ExecutablePath) {
    $requested = (Resolve-Path -LiteralPath $ExecutablePath).Path
    if (-not [String]::Equals($requested, $expected, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'The requested executable is not the development release for this checkout.'
    }
}

. (Join-Path $PSScriptRoot 'launch-dev-guards.ps1')
$running = @(Get-CimInstance Win32_Process -Filter "Name='texpio.exe'")
Assert-TexpioNoRunningProcess -Processes $running

. (Join-Path $PSScriptRoot 'windows-explorer-parent.ps1')
$processId = Start-TexpioExplorerChild -ExecutablePath $expected -WorkingDirectory (Split-Path -Parent $expected)
for ($attempt = 0; $attempt -lt 50; $attempt++) {
    $started = Get-Process -Id $processId -ErrorAction SilentlyContinue
    if (Test-TexpioLaunchReady -Process $started -ExpectedExecutable $expected) {
        Write-Output "Started Texpio PID $processId from $expected using the interactive Explorer parent."
        exit 0
    }
    Start-Sleep -Milliseconds 200
}
throw "Texpio PID $processId started but its window did not open within 10 seconds; verify the process before retrying."

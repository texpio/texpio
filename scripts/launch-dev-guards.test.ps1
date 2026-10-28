$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'launch-dev-guards.ps1')

function Assert-Throws {
    param([scriptblock] $Action, [string] $MessagePattern)
    try {
        & $Action
    } catch {
        if ($_.Exception.Message -notmatch $MessagePattern) { throw }
        return
    }
    throw "Expected an error matching: $MessagePattern"
}

$expected = 'C:\work\texpio\src-tauri\target\release\texpio.exe'
Assert-TexpioNoRunningProcess -Processes @()
Assert-Throws { Assert-TexpioNoRunningProcess -Processes @([pscustomobject]@{ ExecutablePath = $expected }) } 'already running'
Assert-Throws { Assert-TexpioNoRunningProcess -Processes @([pscustomobject]@{ ExecutablePath = 'C:\another\texpio.exe' }) } 'already running'
Assert-Throws { Assert-TexpioNoRunningProcess -Processes @([pscustomobject]@{ ExecutablePath = $null }) } 'already running'
Assert-Throws { Test-TexpioLaunchReady -Process $null -ExpectedExecutable $expected } 'exited'
Assert-Throws { Test-TexpioLaunchReady -Process ([pscustomobject]@{ Path = $null }) -ExpectedExecutable $expected } 'could not be verified'
Assert-Throws { Test-TexpioLaunchReady -Process ([pscustomobject]@{ Path = 'C:\another\texpio.exe' }) -ExpectedExecutable $expected } 'not this checkout'
if (Test-TexpioLaunchReady -Process ([pscustomobject]@{ Path = $expected; MainWindowHandle = [IntPtr]::Zero }) -ExpectedExecutable $expected) {
    throw 'A process without a window must not be reported ready.'
}
if (-not (Test-TexpioLaunchReady -Process ([pscustomobject]@{ Path = $expected.ToUpperInvariant(); MainWindowHandle = [IntPtr]123 }) -ExpectedExecutable $expected)) {
    throw 'The expected process with a window must be reported ready.'
}
Write-Output 'Launcher guard checks passed.'

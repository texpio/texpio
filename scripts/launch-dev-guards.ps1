# Keep the development launcher fail-closed when another installation may own
# Texpio's single-instance mutex. Never stop any process from these checks.
function Assert-TexpioNoRunningProcess {
    param([object[]] $Processes)
    if ($Processes.Count -ne 0) {
        throw 'A Texpio process is already running. Close it before launching this development build; it may otherwise receive the launch instead.'
    }
}

function Test-TexpioLaunchReady {
    param([object] $Process, [string] $ExpectedExecutable)
    if (-not $Process) {
        throw 'The new Texpio process exited before its window opened. Another installation may have received the launch.'
    }
    if (-not $Process.Path) {
        throw 'The new Texpio process path could not be verified.'
    }
    if (-not [String]::Equals($Process.Path, $ExpectedExecutable, [StringComparison]::OrdinalIgnoreCase)) {
        throw "The new process is not this checkout's Texpio executable."
    }
    return $Process.MainWindowHandle -ne [IntPtr]::Zero
}

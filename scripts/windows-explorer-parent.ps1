# Start a process with the interactive Windows shell as its actual parent. This
# avoids inheriting a packaged development host's AppData filesystem view.
if (-not ('Texpio.ExplorerParent' -as [type])) {
    Add-Type @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;

namespace Texpio {
    public static class ExplorerParent {
        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        private struct StartupInfo {
            public int Size;
            public string Reserved, Desktop, Title;
            public uint X, Y, XSize, YSize, XChars, YChars, Fill, Flags;
            public ushort Show, Reserved2;
            public IntPtr Reserved3, StandardInput, StandardOutput, StandardError;
        }
        [StructLayout(LayoutKind.Sequential)]
        private struct StartupInfoEx { public StartupInfo Startup; public IntPtr Attributes; }
        [StructLayout(LayoutKind.Sequential)]
        private struct ProcessInfo { public IntPtr Process, Thread; public uint ProcessId, ThreadId; }

        [DllImport("user32.dll", SetLastError = true)]
        private static extern IntPtr GetShellWindow();
        [DllImport("user32.dll", SetLastError = true)]
        private static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr OpenProcess(uint access, bool inherit, uint processId);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool InitializeProcThreadAttributeList(IntPtr list, int count, int flags, ref IntPtr size);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool UpdateProcThreadAttribute(IntPtr list, uint flags, IntPtr attribute, IntPtr value, IntPtr size, IntPtr previous, IntPtr returned);
        [DllImport("kernel32.dll")]
        private static extern void DeleteProcThreadAttributeList(IntPtr list);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool CreateProcessW(string application, StringBuilder command, IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags, IntPtr environment, string directory, ref StartupInfoEx startup, out ProcessInfo process);
        [DllImport("kernel32.dll")]
        private static extern bool CloseHandle(IntPtr handle);

        public static uint ShellProcessId() {
            IntPtr window = GetShellWindow();
            uint pid;
            if (window == IntPtr.Zero || GetWindowThreadProcessId(window, out pid) == 0 || pid == 0)
                throw new InvalidOperationException("The interactive Windows shell is unavailable.");
            return pid;
        }

        public static uint Launch(uint shellPid, string application, string arguments, string directory, bool hidden) {
            IntPtr parent = OpenProcess(0x80, false, shellPid); // PROCESS_CREATE_PROCESS
            if (parent == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
            IntPtr list = IntPtr.Zero, parentPointer = IntPtr.Zero;
            bool initialized = false;
            try {
                IntPtr size = IntPtr.Zero;
                InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
                if (size == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
                list = Marshal.AllocHGlobal(size);
                if (!InitializeProcThreadAttributeList(list, 1, 0, ref size)) throw new Win32Exception(Marshal.GetLastWin32Error());
                initialized = true;
                parentPointer = Marshal.AllocHGlobal(IntPtr.Size);
                Marshal.WriteIntPtr(parentPointer, parent);
                if (!UpdateProcThreadAttribute(list, 0, new IntPtr(0x20000), parentPointer, new IntPtr(IntPtr.Size), IntPtr.Zero, IntPtr.Zero))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                StartupInfoEx startup = new StartupInfoEx();
                startup.Startup.Size = Marshal.SizeOf(typeof(StartupInfoEx));
                startup.Startup.Flags = 1; // STARTF_USESHOWWINDOW
                startup.Startup.Show = (ushort)(hidden ? 0 : 1); // SW_HIDE or SW_SHOWNORMAL
                startup.Attributes = list;
                string command = "\"" + application + "\"" + (String.IsNullOrEmpty(arguments) ? "" : " " + arguments);
                uint flags = 0x00080000 | (hidden ? 0x08000000u : 0u); // EXTENDED_STARTUPINFO_PRESENT; CREATE_NO_WINDOW for helpers
                ProcessInfo process;
                if (!CreateProcessW(application, new StringBuilder(command), IntPtr.Zero, IntPtr.Zero, false, flags, IntPtr.Zero, directory, ref startup, out process))
                    throw new Win32Exception(Marshal.GetLastWin32Error());
                try { return process.ProcessId; }
                finally { CloseHandle(process.Thread); CloseHandle(process.Process); }
            } finally {
                if (parentPointer != IntPtr.Zero) Marshal.FreeHGlobal(parentPointer);
                if (initialized) DeleteProcThreadAttributeList(list);
                if (list != IntPtr.Zero) Marshal.FreeHGlobal(list);
                CloseHandle(parent);
            }
        }
    }
}
'@
}

function Start-TexpioExplorerChild {
    param(
        [Parameter(Mandatory)] [string] $ExecutablePath,
        [string] $Arguments = '',
        [Parameter(Mandatory)] [string] $WorkingDirectory,
        [switch] $Hidden
    )
    if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
        throw 'The Explorer-parent launcher requires Windows.'
    }
    $application = (Resolve-Path -LiteralPath $ExecutablePath -ErrorAction Stop).Path
    $directory = (Resolve-Path -LiteralPath $WorkingDirectory -ErrorAction Stop).Path
    $texpioShellProcessId = [Texpio.ExplorerParent]::ShellProcessId()
    $shell = Get-Process -Id $texpioShellProcessId -ErrorAction Stop
    $expectedShell = Join-Path ([Environment]::GetFolderPath('Windows')) 'explorer.exe'
    if (-not [String]::Equals($shell.Path, $expectedShell, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'The interactive shell is not the expected Windows Explorer executable.'
    }
    if ($shell.SessionId -ne (Get-Process -Id $PID).SessionId) {
        throw 'Windows Explorer belongs to a different session.'
    }
    [Texpio.ExplorerParent]::Launch($texpioShellProcessId, $application, $Arguments, $directory, $Hidden.IsPresent)
}

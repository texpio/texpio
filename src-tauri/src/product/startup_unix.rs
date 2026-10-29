//! User-owned login entries. Enabling autostart never starts a second process now.
use super::effects::ProductError;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct StartupValue(Option<Vec<u8>>);
fn location(name: &str) -> Result<PathBuf, ProductError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
    {
        return Err(ProductError::InvalidSettings);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or(ProductError::Unavailable)?;
    Ok(if cfg!(target_os = "macos") {
        home.join("Library/LaunchAgents")
            .join(format!("{name}.plist"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(".config"))
            .join("autostart")
            .join(format!("{name}.desktop"))
    })
}
pub fn snapshot(name: &str) -> Result<StartupValue, ProductError> {
    let path = location(name)?;
    match std::fs::read(path) {
        Ok(bytes) if bytes.len() <= 65_536 => Ok(StartupValue(Some(bytes))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(StartupValue(None)),
        _ => Err(ProductError::Unavailable),
    }
}
pub fn restore(name: &str, value: StartupValue) -> Result<(), ProductError> {
    let path = location(name)?;
    if let Some(bytes) = value.0 {
        use std::io::Write;
        let directory = path.parent().ok_or(ProductError::Unavailable)?;
        std::fs::create_dir_all(directory).map_err(|_| ProductError::Unavailable)?;
        let mut file =
            tempfile::NamedTempFile::new_in(directory).map_err(|_| ProductError::Unavailable)?;
        file.write_all(&bytes)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|_| ProductError::Unavailable)?;
        file.persist(path).map_err(|_| ProductError::Unavailable)?;
    } else if let Err(error) = std::fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(ProductError::Unavailable);
    }
    Ok(())
}
pub fn set(name: &str, enabled: bool) -> Result<(), ProductError> {
    let value = if enabled {
        let current = std::env::current_exe().map_err(|_| ProductError::Unavailable)?;
        let image = if cfg!(target_os = "linux") {
            std::env::var_os("APPIMAGE")
        } else {
            None
        };
        let executable = login_executable(&current, image.as_deref())?;
        Some(entry(&executable, cfg!(target_os = "macos"))?.into_bytes())
    } else {
        None
    };
    restore(name, StartupValue(value))
}
fn login_executable(
    current: &Path,
    image: Option<&std::ffi::OsStr>,
) -> Result<PathBuf, ProductError> {
    match image {
        Some(image) if Path::new(image).is_absolute() => Ok(PathBuf::from(image)),
        Some(_) => Err(ProductError::Unavailable),
        None => Ok(current.to_path_buf()),
    }
}
fn entry(executable: &Path, macos: bool) -> Result<String, ProductError> {
    let path = executable
        .to_str()
        .filter(|p| !p.chars().any(char::is_control))
        .ok_or(ProductError::Unavailable)?;
    if macos {
        let xml = path
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");
        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict><key>Label</key><string>io.github.texpio.texpio</string><key>ProgramArguments</key><array><string>{xml}</string><string>--hidden</string></array><key>RunAtLoad</key><true/></dict></plist>\n"
        ))
    } else {
        let exec = path
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('$', "\\$")
            .replace('%', "%%");
        Ok(format!(
            "[Desktop Entry]\nType=Application\nName=Texpio\nExec=\"{exec}\" --hidden\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appimage_login_uses_the_persistent_launcher_not_the_mount() {
        let mounted = std::env::temp_dir().join(".mount_texpio/usr/bin/texpio");
        let persistent = std::env::temp_dir().join("Texpio_X11.AppImage");
        assert_eq!(
            login_executable(&mounted, Some(persistent.as_os_str())).unwrap(),
            persistent
        );
        assert_eq!(login_executable(&mounted, None).unwrap(), mounted);
        assert!(
            login_executable(&mounted, Some(std::ffi::OsStr::new("relative.AppImage"))).is_err()
        );
    }
    #[test]
    fn login_entries_escape_paths_without_shell_expansion() {
        let path = Path::new("/tmp/a & b/texpio%app");
        let mac = entry(path, true).unwrap();
        assert!(mac.contains("/tmp/a &amp; b/texpio%app"));
        assert!(mac.contains("<string>--hidden</string>"));
        let linux = entry(path, false).unwrap();
        assert!(linux.contains("Exec=\"/tmp/a & b/texpio%%app\" --hidden"));
        assert!(!linux.contains("sh -c"));
        assert!(entry(Path::new("/tmp/bad\nname"), false).is_err());
    }
}

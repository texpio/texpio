#[derive(Debug, PartialEq, Eq)]
pub enum CloseBehavior {
    Hide,
    Default,
    Quit,
}
pub fn close_behavior(label: &str, close_to_tray: bool) -> CloseBehavior {
    if label == "main" && !close_to_tray {
        CloseBehavior::Quit
    } else if matches!(label, "main" | "quick-pane") {
        CloseBehavior::Hide
    } else {
        CloseBehavior::Default
    }
}
pub fn hidden_startup(args: impl IntoIterator<Item = std::ffi::OsString>) -> bool {
    args.into_iter().any(|arg| arg == "--hidden")
}
#[cfg(any(target_os = "windows", test))]
pub fn startup_command(path: &std::path::Path) -> Option<String> {
    let path = path.to_str()?;
    if path.contains(['\0', '"']) || !std::path::Path::new(path).is_absolute() {
        return None;
    }
    Some(format!("\"{path}\" --hidden"))
}
pub fn stop_then_exit(stop: impl FnOnce(), exit: impl FnOnce()) {
    stop();
    exit();
}

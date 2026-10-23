#[derive(Debug, PartialEq, Eq)]
pub enum CloseBehavior {
    Hide,
    Default,
}
pub fn close_behavior(label: &str) -> CloseBehavior {
    if matches!(label, "main" | "quick-pane") {
        CloseBehavior::Hide
    } else {
        CloseBehavior::Default
    }
}
pub fn stop_then_exit(stop: impl FnOnce(), exit: impl FnOnce()) {
    stop();
    exit();
}

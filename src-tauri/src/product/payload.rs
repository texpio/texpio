use std::path::PathBuf;

/// Native-owned prepared file paths; never exposed through the webview bridge.
#[derive(Clone, Default)]
pub struct FileDelivery {
    pub files: Vec<PathBuf>,
    pub first: bool,
    pub strict: bool,
}

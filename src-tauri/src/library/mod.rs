//! Rust-owned offline snippet library. Public methods expose data, never SQL.

use serde::{Deserialize, Serialize};
pub mod location;
pub mod watch;
use specta::Type;

mod store;
mod validation;
pub use store::{
    Attachment, BackupFormat, FileSettings, ImportMode, LibraryStore, PreparedImport,
    RecoveryStatus,
};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_SNIPPETS: usize = 10_000;
pub const MAX_FOLDERS: usize = 1_000;
pub const MAX_NAME_SCALARS: usize = 120;
pub const MAX_BODY_SCALARS: usize = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Folder {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snippet {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub trigger: String,
    pub body: String,
    pub enabled: bool,
    pub deleted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnippetInfo {
    pub snippet_id: String,
    pub favorite: bool,
    pub usage_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Library {
    pub schema_version: u32,
    pub revision: u32,
    pub folders: Vec<Folder>,
    pub snippets: Vec<Snippet>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveLibrary {
    pub expected_revision: u32,
    pub folders: Vec<Folder>,
    pub snippets: Vec<Snippet>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportPreview {
    pub folders: u32,
    pub snippets: u32,
    pub skipped: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryError {
    pub code: String,
}

impl LibraryError {
    fn storage() -> Self {
        Self {
            code: "storage".into(),
        }
    }

    fn invalid_data() -> Self {
        Self {
            code: "invalidData".into(),
        }
    }

    fn conflict() -> Self {
        Self {
            code: "conflict".into(),
        }
    }

    fn too_large() -> Self {
        Self {
            code: "tooLarge".into(),
        }
    }

    fn unsupported_version() -> Self {
        Self {
            code: "unsupportedVersion".into(),
        }
    }
}

#[cfg(test)]
mod tests;

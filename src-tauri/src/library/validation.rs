use super::*;
use std::{
    collections::HashSet,
    io::{self, Write},
};

pub(super) fn validate_records(
    folders: &[Folder],
    snippets: &[Snippet],
    require_references: bool,
) -> Result<(), LibraryError> {
    if folders.len() > MAX_FOLDERS || snippets.len() > MAX_SNIPPETS {
        return Err(LibraryError::too_large());
    }
    let mut ids = HashSet::new();
    let mut folder_ids = HashSet::new();
    let mut active_triggers = HashSet::new();
    for folder in folders {
        if !canonical_uuid(&folder.id) || !ids.insert(folder.id.as_str()) {
            return Err(LibraryError::invalid_data());
        }
        validate_name(&folder.name)?;
        folder_ids.insert(folder.id.as_str());
    }
    for snippet in snippets {
        if !canonical_uuid(&snippet.id) || !ids.insert(snippet.id.as_str()) {
            return Err(LibraryError::invalid_data());
        }
        if let Some(folder_id) = &snippet.folder_id
            && (!canonical_uuid(folder_id)
                || (require_references && !folder_ids.contains(folder_id.as_str())))
        {
            return Err(LibraryError::invalid_data());
        }
        validate_name(&snippet.title)?;
        if !crate::product::matcher::valid_trigger(&snippet.trigger)
            || snippet
                .body
                .chars()
                .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
        {
            return Err(LibraryError::invalid_data());
        }
        if snippet.body.chars().take(MAX_BODY_SCALARS + 1).count() > MAX_BODY_SCALARS {
            return Err(LibraryError::too_large());
        }
        if snippet.enabled && !snippet.deleted && !active_triggers.insert(snippet.trigger.as_str())
        {
            return Err(LibraryError::conflict());
        }
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), LibraryError> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(LibraryError::invalid_data());
    }
    if name.chars().take(MAX_NAME_SCALARS + 1).count() > MAX_NAME_SCALARS {
        return Err(LibraryError::too_large());
    }
    Ok(())
}

pub(super) fn canonical_uuid(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportEnvelope {
    schema_version: u32,
    #[serde(default)]
    revision: u32,
    folders: Vec<Folder>,
    snippets: Vec<Snippet>,
}

pub(super) fn parse_import(json: &str) -> Result<Library, LibraryError> {
    if json.len() > MAX_JSON_BYTES {
        return Err(LibraryError::too_large());
    }
    let envelope: ImportEnvelope =
        serde_json::from_str(json).map_err(|_| LibraryError::invalid_data())?;
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(LibraryError::unsupported_version());
    }
    validate_records(&envelope.folders, &envelope.snippets, false)?;
    Ok(Library {
        schema_version: SCHEMA_VERSION,
        revision: envelope.revision,
        folders: envelope.folders,
        snippets: envelope.snippets,
    })
}

/// A bounded writer keeps even a library of individually valid large records
/// from allocating an unbounded export before the JSON limit can be checked.
struct BoundedJson {
    bytes: Vec<u8>,
    overflow: bool,
}

impl Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_JSON_BYTES.saturating_sub(self.bytes.len()) {
            self.overflow = true;
            return Err(io::Error::other("library size limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn encode_library(library: &Library) -> Result<String, LibraryError> {
    let mut writer = BoundedJson {
        bytes: Vec::new(),
        overflow: false,
    };
    if serde_json::to_writer(&mut writer, library).is_err() {
        return Err(if writer.overflow {
            LibraryError::too_large()
        } else {
            LibraryError::storage()
        });
    }
    String::from_utf8(writer.bytes).map_err(|_| LibraryError::storage())
}

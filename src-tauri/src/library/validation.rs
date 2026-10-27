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
        if let Some(document) = &snippet.rich_body {
            if document.len() > MAX_RICH_BODY_BYTES {
                return Err(LibraryError::too_large());
            }
            super::json_strict::decode(document.as_bytes())
                .map_err(|_| LibraryError::invalid_data())?;
            let rendered =
                crate::rich_text::render(document).map_err(|_| LibraryError::invalid_data())?;
            if rendered.plain != snippet.body {
                return Err(LibraryError::invalid_data());
            }
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
    let value =
        super::json_strict::decode(json.as_bytes()).map_err(|_| LibraryError::invalid_data())?;
    let envelope: ImportEnvelope =
        serde_json::from_value(value.clone()).map_err(|_| LibraryError::invalid_data())?;
    validate_interchange_version(&value, envelope.schema_version)?;
    validate_records(&envelope.folders, &envelope.snippets, false)?;
    Ok(Library {
        schema_version: SCHEMA_VERSION,
        revision: envelope.revision,
        folders: envelope.folders,
        snippets: envelope.snippets,
    })
}

pub(super) fn interchange_version(snippets: &[Snippet]) -> u32 {
    if snippets.iter().any(|snippet| snippet.rich_body.is_some()) {
        RICH_SCHEMA_VERSION
    } else {
        SCHEMA_VERSION
    }
}

pub(super) fn validate_interchange_version(
    value: &serde_json::Value,
    version: u32,
) -> Result<(), LibraryError> {
    if !matches!(version, SCHEMA_VERSION | RICH_SCHEMA_VERSION) {
        return Err(LibraryError::unsupported_version());
    }
    if version == SCHEMA_VERSION
        && value
            .get("snippets")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|snippets| {
                snippets
                    .iter()
                    .any(|snippet| snippet.get("richBody").is_some())
            })
    {
        return Err(LibraryError::invalid_data());
    }
    Ok(())
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
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ExportLibrary<'a> {
        schema_version: u32,
        revision: u32,
        folders: &'a [Folder],
        snippets: &'a [Snippet],
    }
    let export = ExportLibrary {
        schema_version: interchange_version(&library.snippets),
        revision: library.revision,
        folders: &library.folders,
        snippets: &library.snippets,
    };
    let mut writer = BoundedJson {
        bytes: Vec::new(),
        overflow: false,
    };
    if serde_json::to_writer(&mut writer, &export).is_err() {
        return Err(if writer.overflow {
            LibraryError::too_large()
        } else {
            LibraryError::storage()
        });
    }
    String::from_utf8(writer.bytes).map_err(|_| LibraryError::storage())
}

#[cfg(test)]
mod duplicate_key_tests {
    use super::*;

    #[test]
    fn bare_import_rejects_duplicate_keys_at_root_and_nested_record() {
        let root = r#"{"schemaVersion":1,"schemaVersion":1,"folders":[],"snippets":[]}"#;
        let nested = r#"{"schemaVersion":1,"folders":[{"id":"10000000-0000-4000-8000-000000000000","name":"A","name":"B"}],"snippets":[]}"#;
        for json in [root, nested] {
            assert_eq!(parse_import(json).unwrap_err().code, "invalidData");
        }
    }

    #[test]
    fn embedded_rich_document_rejects_duplicate_attribute_keys() {
        let rich = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"https://one.example","href":"https://two.example"}}]}]}]}}"#;
        let json = serde_json::json!({
            "schemaVersion": RICH_SCHEMA_VERSION,
            "folders": [],
            "snippets": [{
                "id": "20000000-0000-4000-8000-000000000000", "folderId": null,
                "title": "Example", "trigger": ";x", "body": "x", "richBody": rich,
                "enabled": true, "deleted": false
            }]
        })
        .to_string();
        assert_eq!(parse_import(&json).unwrap_err().code, "invalidData");
    }
}

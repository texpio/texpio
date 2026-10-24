use super::*;
#[path = "yaml_budget.rs"]
mod yaml_budget;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    io::Cursor,
};
const MAX_MANIFEST: usize = 16 * 1024 * 1024;
const MAX_BLOBS: usize = 512 * 1024 * 1024;
pub const MAX_ARCHIVE: usize = MAX_BLOBS + MAX_MANIFEST + 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum BackupFormat {
    Json,
    Yaml,
    Archive,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ImportMode {
    Merge,
    Replace,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FileOptions {
    snippet_id: String,
    settings: FileSettings,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    library: Library,
    #[serde(default)]
    metadata: Vec<SnippetInfo>,
    #[serde(default)]
    attachments: Vec<Attachment>,
    #[serde(default)]
    file_settings: Vec<FileOptions>,
}
pub struct PreparedImport {
    manifest: Manifest,
    blobs: BTreeMap<String, Vec<u8>>,
}
impl PreparedImport {
    pub fn decode(data: &[u8], format: BackupFormat) -> Result<Self, LibraryError> {
        if data.len()
            > if matches!(format, BackupFormat::Archive) {
                MAX_ARCHIVE
            } else {
                MAX_MANIFEST
            }
        {
            return Err(LibraryError::too_large());
        }
        let (manifest, blobs) = if matches!(format, BackupFormat::Archive) {
            decode_archive(data)?
        } else {
            (parse_manifest(data, format)?, BTreeMap::new())
        };
        let result = Self { manifest, blobs };
        result.validate()?;
        Ok(result)
    }
    fn validate(&self) -> Result<(), LibraryError> {
        let manifest = &self.manifest;
        if manifest.schema_version != 2 || manifest.library.schema_version != SCHEMA_VERSION {
            return Err(LibraryError::unsupported_version());
        }
        validate_records(&manifest.library.folders, &manifest.library.snippets, true)?;
        encode_library(&manifest.library)?;
        let snippets: HashSet<_> = manifest
            .library
            .snippets
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        if manifest.metadata.len() > MAX_SNIPPETS
            || manifest.file_settings.len() > MAX_SNIPPETS
            || manifest.attachments.len() > 100_000
        {
            return Err(LibraryError::too_large());
        }
        let mut ids = HashSet::new();
        for info in &manifest.metadata {
            if !snippets.contains(info.snippet_id.as_str())
                || !ids.insert(&info.snippet_id)
                || info.created_at < 0
                || info.updated_at < 0
                || info.last_used_at.is_some_and(|time| time < 0)
            {
                return Err(LibraryError::invalid_data());
            }
        }
        ids.clear();
        for options in &manifest.file_settings {
            if !snippets.contains(options.snippet_id.as_str()) || !ids.insert(&options.snippet_id) {
                return Err(LibraryError::invalid_data());
            }
        }
        ids.clear();
        let mut positions = HashSet::new();
        let mut counts = HashMap::<&str, usize>::new();
        let mut wanted = HashSet::new();
        let mut total = 0usize;
        for file in &manifest.attachments {
            if !super::super::validation::canonical_uuid(&file.id)
                || !ids.insert(&file.id)
                || !snippets.contains(file.snippet_id.as_str())
                || !attachments::valid_leaf(&file.name)
                || !attachments::valid_digest(&file.digest)
                || file.bytes > 128 * 1024 * 1024
                || !positions.insert((&file.snippet_id, file.position))
            {
                return Err(LibraryError::invalid_data());
            }
            let count = counts.entry(&file.snippet_id).or_default();
            *count += 1;
            if *count > 100 {
                return Err(LibraryError::too_large());
            }
            total = total
                .checked_add(file.bytes as usize)
                .ok_or_else(LibraryError::too_large)?;
            if total > MAX_BLOBS {
                return Err(LibraryError::too_large());
            }
            let blob = self
                .blobs
                .get(&file.digest)
                .ok_or_else(LibraryError::invalid_data)?;
            if blob.len() as u64 != file.bytes
                || format!("{:x}", Sha256::digest(blob)) != file.digest
            {
                return Err(LibraryError::invalid_data());
            }
            wanted.insert(file.digest.as_str());
        }
        if wanted.len() != self.blobs.len() {
            return Err(LibraryError::invalid_data());
        }
        Ok(())
    }
    pub fn preview(
        &self,
        store: &LibraryStore,
        mode: ImportMode,
    ) -> Result<(ImportPreview, u32), LibraryError> {
        let current = store.load()?;
        let (library, preview) = self.merged(&current, mode)?;
        encode_library(&library)?;
        Ok((preview, current.revision))
    }
    pub fn file_count(&self) -> u32 {
        self.manifest.attachments.len() as u32
    }
    fn merged(
        &self,
        current: &Library,
        mode: ImportMode,
    ) -> Result<(Library, ImportPreview), LibraryError> {
        match mode {
            ImportMode::Merge => merge_import(current, self.manifest.library.clone()),
            ImportMode::Replace => Ok((
                self.manifest.library.clone(),
                ImportPreview {
                    folders: self.manifest.library.folders.len() as u32,
                    snippets: self.manifest.library.snippets.len() as u32,
                    skipped: 0,
                },
            )),
        }
    }
    pub fn apply(
        &self,
        store: &LibraryStore,
        revision: u32,
        mode: ImportMode,
    ) -> Result<Library, LibraryError> {
        self.validate()?;
        let _guard = store
            .operation
            .lock()
            .map_err(|_| LibraryError::storage())?;
        let mut connection = store.open()?;
        // VACUUM INTO cannot run inside a transaction. Validate before making
        // the snapshot, then recheck the revision under the writer lock below.
        let current = read_library(&connection)?;
        if current.revision != revision {
            return Err(LibraryError::conflict());
        }
        let (candidate, _) = self.merged(&current, mode)?;
        encode_library(&candidate)?;
        store.backup_database(&connection)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let previous = read_library(&transaction)?;
        if previous.revision != revision {
            return Err(LibraryError::conflict());
        }
        let (mut library, _) = self.merged(&previous, mode)?;
        if matches!(mode, ImportMode::Merge) {
            let current_bytes: i64 = transaction
                .query_row(
                    "SELECT coalesce(sum(bytes),0) FROM attachments",
                    [],
                    |row| row.get(0),
                )
                .map_err(|_| LibraryError::storage())?;
            let previous_ids: HashSet<_> =
                previous.snippets.iter().map(|s| s.id.as_str()).collect();
            let incoming_bytes: u64 = self
                .manifest
                .attachments
                .iter()
                .filter(|file| !previous_ids.contains(file.snippet_id.as_str()))
                .map(|file| file.bytes)
                .sum();
            if current_bytes < 0
                || (current_bytes as u64).saturating_add(incoming_bytes) > MAX_BLOBS as u64
            {
                return Err(LibraryError::too_large());
            }
        }
        library.revision = revision.checked_add(1).ok_or_else(LibraryError::conflict)?;
        encode_library(&library)?;
        // A database snapshot includes metadata/file references. Blob storage is immutable.
        let directory = store.root.join("attachments");
        fs::create_dir_all(&directory).map_err(|_| LibraryError::storage())?;
        for (digest, bytes) in &self.blobs {
            let destination = directory.join(digest);
            if destination.exists() {
                let metadata =
                    fs::symlink_metadata(&destination).map_err(|_| LibraryError::storage())?;
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || metadata.len() != bytes.len() as u64
                {
                    return Err(LibraryError::storage());
                }
                let existing = fs::read(&destination).map_err(|_| LibraryError::storage())?;
                if Sha256::digest(&existing) != Sha256::digest(bytes) {
                    return Err(LibraryError::storage());
                }
            } else {
                let mut temp = tempfile::NamedTempFile::new_in(&directory)
                    .map_err(|_| LibraryError::storage())?;
                temp.write_all(bytes).map_err(|_| LibraryError::storage())?;
                temp.as_file()
                    .sync_all()
                    .map_err(|_| LibraryError::storage())?;
                temp.persist_noclobber(destination)
                    .map_err(|_| LibraryError::storage())?;
            }
        }
        if matches!(mode, ImportMode::Replace) {
            transaction
                .execute_batch(
                    "DELETE FROM snippet_info; DELETE FROM attachments; DELETE FROM file_settings;",
                )
                .map_err(|_| LibraryError::storage())?;
        }
        replace_library(&transaction, &library, Some(&previous))?;
        let previous_ids: HashSet<_> = previous.snippets.iter().map(|s| s.id.as_str()).collect();
        let incoming: HashSet<_> = self
            .manifest
            .library
            .snippets
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        let apply_id = |id: &str| matches!(mode, ImportMode::Replace) || !previous_ids.contains(id);
        for info in &self.manifest.metadata {
            if apply_id(&info.snippet_id) {
                transaction.execute("UPDATE snippet_info SET favorite=?1,usage_count=?2,created_at=?3,updated_at=?4,last_used_at=?5 WHERE id=?6", params![info.favorite,info.usage_count,info.created_at,info.updated_at,info.last_used_at,info.snippet_id]).map_err(|_| LibraryError::storage())?;
            }
        }
        for file in &self.manifest.attachments {
            if apply_id(&file.snippet_id) {
                transaction.execute("INSERT INTO attachments(id,snippet_id,name,digest,bytes,position) VALUES(?1,?2,?3,?4,?5,?6)",params![file.id,file.snippet_id,file.name,file.digest,file.bytes as i64,file.position]).map_err(|_| LibraryError::conflict())?;
            }
        }
        for options in &self.manifest.file_settings {
            if incoming.contains(options.snippet_id.as_str()) && apply_id(&options.snippet_id) {
                transaction.execute("INSERT INTO file_settings(snippet_id,attachments_first,strict_order) VALUES(?1,?2,?3) ON CONFLICT(snippet_id) DO UPDATE SET attachments_first=excluded.attachments_first,strict_order=excluded.strict_order",params![options.snippet_id,options.settings.attachments_first,options.settings.strict_order]).map_err(|_| LibraryError::storage())?;
            }
        }
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(library)
    }
}
impl LibraryStore {
    fn backup_database(&self, connection: &Connection) -> Result<(), LibraryError> {
        let directory = self.root.join("backups");
        fs::create_dir_all(&directory).map_err(|_| LibraryError::storage())?;
        let path = directory.join(format!(
            "library-before-import-{}.sqlite3",
            uuid::Uuid::new_v4()
        ));
        let temporary = tempfile::NamedTempFile::new_in(&directory)
            .map_err(|_| LibraryError::storage())?
            .into_temp_path();
        // SQLite makes a consistent snapshot including committed WAL content.
        connection
            .execute("VACUUM INTO ?1", [temporary.to_string_lossy().as_ref()])
            .map_err(|_| LibraryError::storage())?;
        temporary
            .persist_noclobber(path)
            .map_err(|_| LibraryError::storage())?;
        Ok(())
    }
    pub fn export_backup(&self, format: BackupFormat) -> Result<Vec<u8>, LibraryError> {
        // All reads use one transaction; another instance cannot mix revisions.
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut connection = self.open()?;
        let transaction = connection
            .transaction()
            .map_err(|_| LibraryError::storage())?;
        let library = read_library(&transaction)?;
        let mut statement = transaction.prepare("SELECT id,favorite,usage_count,created_at,updated_at,last_used_at FROM snippet_info ORDER BY id LIMIT 10001").map_err(|_| LibraryError::storage())?;
        let metadata = statement
            .query_map([], |r| {
                Ok(SnippetInfo {
                    snippet_id: r.get(0)?,
                    favorite: r.get(1)?,
                    usage_count: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                    last_used_at: r.get(5)?,
                })
            })
            .map_err(|_| LibraryError::storage())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| LibraryError::storage())?;
        drop(statement);
        let mut attachments = Vec::new();
        let mut blobs = BTreeMap::new();
        let mut file_settings = Vec::new();
        let mut total = 0usize;
        if matches!(format, BackupFormat::Archive) {
            let mut statement = transaction.prepare("SELECT id,snippet_id,name,digest,bytes,position FROM attachments ORDER BY snippet_id,position LIMIT 100001").map_err(|_| LibraryError::storage())?;
            attachments = statement
                .query_map([], |r| {
                    Ok(Attachment {
                        id: r.get(0)?,
                        snippet_id: r.get(1)?,
                        name: r.get(2)?,
                        digest: r.get(3)?,
                        bytes: u64::from(r.get::<_, u32>(4)?),
                        position: r.get(5)?,
                        present: true,
                    })
                })
                .map_err(|_| LibraryError::storage())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| LibraryError::storage())?;
            for file in &attachments {
                if !attachments::valid_digest(&file.digest) || file.bytes > 128 * 1024 * 1024 {
                    return Err(LibraryError::invalid_data());
                }
                if blobs.contains_key(&file.digest) {
                    continue;
                }
                total = total
                    .checked_add(file.bytes as usize)
                    .ok_or_else(LibraryError::too_large)?;
                if total > MAX_BLOBS {
                    return Err(LibraryError::too_large());
                }
                let path = self.root.join("attachments").join(&file.digest);
                let metadata = fs::symlink_metadata(&path).map_err(|_| LibraryError::storage())?;
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || metadata.len() != file.bytes
                {
                    return Err(LibraryError::storage());
                }
                let mut data = Vec::new();
                fs::File::open(path)
                    .map_err(|_| LibraryError::storage())?
                    .take(file.bytes + 1)
                    .read_to_end(&mut data)
                    .map_err(|_| LibraryError::storage())?;
                if data.len() as u64 != file.bytes
                    || format!("{:x}", Sha256::digest(&data)) != file.digest
                {
                    return Err(LibraryError::storage());
                }
                blobs.insert(file.digest.clone(), data);
            }
            let mut statement = transaction.prepare("SELECT snippet_id,attachments_first,strict_order FROM file_settings ORDER BY snippet_id LIMIT 10001").map_err(|_| LibraryError::storage())?;
            file_settings = statement
                .query_map([], |r| {
                    Ok(FileOptions {
                        snippet_id: r.get(0)?,
                        settings: FileSettings {
                            attachments_first: r.get(1)?,
                            strict_order: r.get(2)?,
                        },
                    })
                })
                .map_err(|_| LibraryError::storage())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| LibraryError::storage())?;
        }
        let prepared = PreparedImport {
            manifest: Manifest {
                schema_version: 2,
                library,
                metadata,
                attachments,
                file_settings,
            },
            blobs,
        };
        prepared.validate()?;
        let json = serde_json::to_vec(&prepared.manifest).map_err(|_| LibraryError::storage())?;
        if json.len() > MAX_MANIFEST {
            return Err(LibraryError::too_large());
        }
        match format {
            BackupFormat::Json => Ok(json),
            BackupFormat::Yaml => {
                let yaml = serde_yaml_ng::to_string(&prepared.manifest)
                    .map_err(|_| LibraryError::storage())?;
                if yaml.len() > MAX_MANIFEST {
                    return Err(LibraryError::too_large());
                }
                Ok(yaml.into_bytes())
            }
            BackupFormat::Archive => {
                let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                archive
                    .start_file("manifest.json", options)
                    .map_err(|_| LibraryError::storage())?;
                archive
                    .write_all(&json)
                    .map_err(|_| LibraryError::storage())?;
                for (digest, data) in prepared.blobs {
                    archive
                        .start_file(format!("blobs/{digest}"), options)
                        .map_err(|_| LibraryError::storage())?;
                    archive
                        .write_all(&data)
                        .map_err(|_| LibraryError::storage())?;
                }
                let bytes = archive
                    .finish()
                    .map_err(|_| LibraryError::storage())?
                    .into_inner();
                if bytes.len() > MAX_ARCHIVE {
                    return Err(LibraryError::too_large());
                }
                Ok(bytes)
            }
        }
    }
}
fn parse_manifest(data: &[u8], format: BackupFormat) -> Result<Manifest, LibraryError> {
    let value: serde_json::Value = if matches!(format, BackupFormat::Yaml) {
        yaml_value(data, MAX_MANIFEST)?
    } else {
        serde_json::from_slice(data).map_err(|_| LibraryError::invalid_data())?
    };
    if value.get("schemaVersion").and_then(|v| v.as_u64()) == Some(1) {
        let json = serde_json::to_string(&value).map_err(|_| LibraryError::invalid_data())?;
        let library = parse_import(&json)?;
        return Ok(Manifest {
            schema_version: 2,
            library,
            metadata: vec![],
            attachments: vec![],
            file_settings: vec![],
        });
    }
    serde_json::from_value(value).map_err(|_| LibraryError::invalid_data())
}
fn yaml_value(data: &[u8], maximum: usize) -> Result<serde_json::Value, LibraryError> {
    yaml_budget::decode(data, maximum).map_err(|_| LibraryError::invalid_data())
}
fn read_archive_entry(
    reader: &mut impl Read,
    declared: u64,
    limit: usize,
    remaining: &mut usize,
) -> Result<Vec<u8>, LibraryError> {
    if declared > limit as u64 || declared > *remaining as u64 {
        return Err(LibraryError::too_large());
    }
    // A false small declaration is rejected after at most one extra byte, rather
    // than allowing that stream to consume the whole per-entry allocation.
    let read_limit = limit.min(*remaining).min(declared as usize);
    let mut bytes = Vec::new();
    reader
        .take(read_limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| LibraryError::invalid_data())?;
    if bytes.len() as u64 != declared {
        return Err(LibraryError::invalid_data());
    }
    *remaining -= bytes.len();
    Ok(bytes)
}
fn decode_archive(data: &[u8]) -> Result<(Manifest, BTreeMap<String, Vec<u8>>), LibraryError> {
    validate_directory(data)?;
    let mut archive =
        zip::ZipArchive::new(Cursor::new(data)).map_err(|_| LibraryError::invalid_data())?;
    if archive.len() > 10001 {
        return Err(LibraryError::too_large());
    }
    let mut manifest = None;
    let mut blobs = BTreeMap::new();
    let mut remaining = MAX_BLOBS + MAX_MANIFEST;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|_| LibraryError::invalid_data())?;
        let name = file.name().to_owned();
        let limit = if name == "manifest.json" {
            MAX_MANIFEST
        } else {
            128 * 1024 * 1024
        };
        if file.size() > limit as u64
            || file.is_dir()
            || file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(LibraryError::invalid_data());
        }
        let declared = file.size();
        let bytes = read_archive_entry(&mut file, declared, limit, &mut remaining)?;
        if name == "manifest.json" {
            manifest = Some(parse_manifest(&bytes, BackupFormat::Json)?);
        } else {
            let digest = name
                .strip_prefix("blobs/")
                .ok_or_else(LibraryError::invalid_data)?;
            if !attachments::valid_digest(digest)
                || blobs.insert(digest.to_owned(), bytes).is_some()
            {
                return Err(LibraryError::invalid_data());
            }
        }
    }
    Ok((manifest.ok_or_else(LibraryError::invalid_data)?, blobs))
}
// Require one ordinary single-disk central directory. No extraction API or ZIP64.
// Checking raw central names also catches duplicates that zip's map would hide.
fn validate_directory(data: &[u8]) -> Result<(), LibraryError> {
    let end = (data.len().saturating_sub(65557)..data.len().saturating_sub(21))
        .rev()
        .find(|&i| {
            data.get(i..i + 4) == Some(b"PK\x05\x06")
                && data.get(i + 20..i + 22).is_some_and(|b| {
                    i + 22 + u16::from_le_bytes([b[0], b[1]]) as usize == data.len()
                })
        })
        .ok_or_else(LibraryError::invalid_data)?;
    let u16at = |i: usize| u16::from_le_bytes([data[i], data[i + 1]]) as usize;
    let u32at = |i: usize| u32::from_le_bytes(data[i..i + 4].try_into().unwrap()) as usize;
    let count = u16at(end + 10);
    if u16at(end + 4) != 0
        || u16at(end + 6) != 0
        || u16at(end + 8) != count
        || count == 0
        || count > 10001
    {
        return Err(LibraryError::invalid_data());
    }
    let mut position = u32at(end + 16);
    let finish = position
        .checked_add(u32at(end + 12))
        .ok_or_else(LibraryError::invalid_data)?;
    if finish != end {
        return Err(LibraryError::invalid_data());
    }
    let mut names = HashSet::new();
    for _ in 0..count {
        if position.checked_add(46).is_none_or(|p| p > finish)
            || data.get(position..position + 4) != Some(b"PK\x01\x02")
        {
            return Err(LibraryError::invalid_data());
        }
        let name_len = u16at(position + 28);
        let extra = u16at(position + 30);
        let comment = u16at(position + 32);
        let next = position
            .checked_add(46 + name_len + extra + comment)
            .filter(|p| *p <= finish)
            .ok_or_else(LibraryError::invalid_data)?;
        if u16at(position + 8) & 1 != 0
            || !matches!(u16at(position + 10), 0 | 8)
            || u16at(position + 34) != 0
        {
            return Err(LibraryError::invalid_data());
        }
        let name = std::str::from_utf8(&data[position + 46..position + 46 + name_len])
            .map_err(|_| LibraryError::invalid_data())?;
        if !(name == "manifest.json"
            || name
                .strip_prefix("blobs/")
                .is_some_and(attachments::valid_digest))
            || !names.insert(name)
        {
            return Err(LibraryError::invalid_data());
        }
        position = next;
    }
    if position != finish {
        return Err(LibraryError::invalid_data());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yaml_alias_expansion_is_bounded_and_duplicate_keys_are_rejected() {
        assert!(yaml_value(b"a: &x 'long long long'\nb: *x\nc: *x\n", 25).is_err());
        assert!(yaml_value(b"same: 1\nsame: 2\n", 1024).is_err());
    }
    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    #[test]
    fn zip_traversal_hidden_duplicates_and_declared_size_abuse_are_rejected() {
        assert!(
            PreparedImport::decode(&archive(&[("../escape", b"x")]), BackupFormat::Archive)
                .is_err()
        );
        assert!(
            PreparedImport::decode(
                &archive(&[("blobs/../../escape", b"x")]),
                BackupFormat::Archive
            )
            .is_err()
        );
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let manifest = store.export_backup(BackupFormat::Json).unwrap();
        let mut duplicate = archive(&[("manifest.json", &manifest), ("manifesz.json", &manifest)]);
        for start in 0..duplicate.len() - 13 {
            if &duplicate[start..start + 13] == b"manifesz.json" {
                duplicate[start..start + 13].copy_from_slice(b"manifest.json");
            }
        }
        assert!(PreparedImport::decode(&duplicate, BackupFormat::Archive).is_err());
        let mut large = archive(&[("manifest.json", &manifest)]);
        let central = large
            .windows(4)
            .position(|bytes| bytes == b"PK\x01\x02")
            .unwrap();
        large[central + 24..central + 28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(PreparedImport::decode(&large, BackupFormat::Archive).is_err());
    }
    #[test]
    fn zip_rejects_false_uncompressed_sizes_even_with_valid_deflate_and_crc() {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let manifest = store.export_backup(BackupFormat::Json).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "manifest.json",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(&manifest).unwrap();
        let original = writer.finish().unwrap().into_inner();
        for declared in [1, manifest.len() as u32 + 1] {
            let mut malformed = original.clone();
            let central = malformed
                .windows(4)
                .position(|bytes| bytes == b"PK\x01\x02")
                .unwrap();
            malformed[22..26].copy_from_slice(&declared.to_le_bytes());
            malformed[central + 24..central + 28].copy_from_slice(&declared.to_le_bytes());
            // The dependency accepts this complete stream and its original CRC.
            // Size declarations therefore cannot stand in for actual output bounds.
            let mut archive = zip::ZipArchive::new(Cursor::new(&malformed)).unwrap();
            let mut file = archive.by_index(0).unwrap();
            assert_eq!(file.size(), u64::from(declared));
            let mut actual = Vec::new();
            file.read_to_end(&mut actual).unwrap();
            assert_eq!(actual, manifest);
            assert!(decode_archive(&malformed).is_err());
        }
    }
    #[test]
    fn archive_reader_limits_streaming_and_debits_actual_remaining_bytes() {
        let mut remaining = 5;
        let mut dishonest = Cursor::new(b"12345678");
        assert!(read_archive_entry(&mut dishonest, 4, 128, &mut remaining).is_err());
        assert_eq!(dishonest.position(), 5);
        assert_eq!(remaining, 5);

        let mut first = Cursor::new(b"1234");
        assert_eq!(
            read_archive_entry(&mut first, 4, 128, &mut remaining).unwrap(),
            b"1234"
        );
        assert_eq!(remaining, 1);
        let mut next = Cursor::new(b"12");
        assert!(read_archive_entry(&mut next, 2, 128, &mut remaining).is_err());
        assert_eq!(next.position(), 0);
        assert_eq!(remaining, 1);
    }
    #[test]
    fn json_yaml_and_archive_round_trip_text_metadata_and_archive_file_bytes() {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let mut library = store.load().unwrap();
        let id = library.snippets[0].id.clone();
        library.snippets[0].body = "literal: [नमस्ते]\n\t👋 & anchor".into();
        let library = store
            .save(SaveLibrary {
                expected_revision: library.revision,
                folders: library.folders,
                snippets: library.snippets,
            })
            .unwrap();
        store.set_favorite(&id, true).unwrap();
        store.record_usage(&id).unwrap();
        store
            .add_attachment_data(&id, "fixture.txt", b"owned attachment\0bytes")
            .unwrap();
        store
            .set_file_settings(
                &id,
                &FileSettings {
                    attachments_first: true,
                    strict_order: true,
                },
            )
            .unwrap();
        for format in [
            BackupFormat::Json,
            BackupFormat::Yaml,
            BackupFormat::Archive,
        ] {
            let bytes = store.export_backup(format).unwrap();
            let parsed = PreparedImport::decode(&bytes, format).unwrap();
            let destination = tempfile::tempdir().unwrap();
            let other = LibraryStore::new(destination.path().into());
            let current = other.load().unwrap();
            let restored = parsed
                .apply(&other, current.revision, ImportMode::Replace)
                .unwrap();
            assert_eq!(restored.snippets, library.snippets);
            assert_eq!(restored.folders, library.folders);
            let metadata = other.metadata().unwrap();
            assert!(
                metadata
                    .iter()
                    .any(|info| info.snippet_id == id && info.favorite && info.usage_count == 1)
            );
            if matches!(format, BackupFormat::Archive) {
                let files = other.delivery_files(&id).unwrap();
                assert_eq!(files.len(), 1);
                assert_eq!(fs::read(&files[0]).unwrap(), b"owned attachment\0bytes");
                assert!(other.file_settings(&id).unwrap().strict_order);
            }
        }
    }
    #[test]
    fn replace_is_explicit_and_stale_or_conflicting_import_preserves_library() {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let original = store.load().unwrap();
        let bytes = store.export_backup(BackupFormat::Json).unwrap();
        let prepared = PreparedImport::decode(&bytes, BackupFormat::Json).unwrap();
        let mut changed = original.clone();
        changed.snippets[0].body = "different body".into();
        let saved = store
            .save(SaveLibrary {
                expected_revision: original.revision,
                folders: changed.folders,
                snippets: changed.snippets,
            })
            .unwrap();
        assert_eq!(
            prepared
                .apply(&store, saved.revision, ImportMode::Merge)
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(store.load().unwrap(), saved);
        assert_eq!(
            prepared
                .apply(&store, original.revision, ImportMode::Replace)
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(store.load().unwrap(), saved);
        assert_eq!(
            prepared
                .apply(&store, saved.revision, ImportMode::Replace)
                .unwrap()
                .snippets,
            original.snippets
        );
    }
    #[test]
    fn legacy_json_is_accepted_and_invalid_archive_never_changes_destination() {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let original = store.load().unwrap();
        assert!(
            PreparedImport::decode(store.export_json().unwrap().as_bytes(), BackupFormat::Json)
                .is_ok()
        );
        assert!(PreparedImport::decode(b"not a zip", BackupFormat::Archive).is_err());
        assert!(PreparedImport::decode(b"schemaVersion: 999\n", BackupFormat::Yaml).is_err());
        assert_eq!(store.load().unwrap(), original);
    }
}

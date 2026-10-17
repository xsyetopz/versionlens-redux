use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Result};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentRecord {
    pub value: serde_json::Value,
    pub attempted_at_ms: u64,
    pub succeeded_at_ms: Option<u64>,
    pub expires_at_ms: u64,
    pub retry_at_ms: Option<u64>,
    pub accessed_at_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Envelope {
    schema: u32,
    epoch: u64,
    #[serde(default)]
    revision: u64,
    records: BTreeMap<String, PersistentRecord>,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            epoch: 0,
            revision: 0,
            records: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    length: u64,
    modified: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SnapshotToken {
    epoch: u64,
    revision: u64,
    cache: Option<FileStamp>,
}

#[derive(Debug)]
struct Snapshot {
    token: SnapshotToken,
    data: Envelope,
}

struct InstallationSecret {
    bytes: [u8; 32],
    replaced: bool,
}

/// A process-shared cache. Keys must be opaque identifiers without credentials.
pub struct PersistentCache {
    directory: PathBuf,
    signing_key: ring::hmac::Key,
    max_records: usize,
    max_bytes: usize,
    snapshot: Mutex<Option<Snapshot>>,
}

impl PersistentCache {
    pub fn open(directory: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(directory.as_ref())?;
        let _lock = Self::lock(directory.as_ref())?;
        let secret = installation_secret(directory.as_ref())?;
        let cache = Self {
            directory: directory.as_ref().to_owned(),
            signing_key: ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &secret.bytes),
            max_records: 10_000,
            max_bytes: 256 * 1024 * 1024,
            snapshot: Mutex::new(None),
        };
        if secret.replaced {
            cache.reset(cache.previous_epoch()?)?;
        }
        Ok(cache)
    }

    #[must_use]
    pub fn with_limits(mut self, max_records: usize, max_bytes: usize) -> Self {
        self.max_records = max_records;
        self.max_bytes = max_bytes;
        self
    }

    pub fn partition_key(&self, components: &[&[u8]]) -> String {
        let mut context = ring::hmac::Context::with_key(&self.signing_key);
        for component in components {
            context.update(
                &u64::try_from(component.len())
                    .unwrap_or(u64::MAX)
                    .to_le_bytes(),
            );
            context.update(component);
        }
        context
            .sign()
            .as_ref()
            .iter()
            .flat_map(|byte| {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                [
                    char::from(HEX[usize::from(byte >> 4)]),
                    char::from(HEX[usize::from(byte & 15)]),
                ]
            })
            .collect()
    }

    pub fn epoch(&self) -> Result<u64> {
        if let Some(epoch) = read_number(&self.directory.join("generation.json"))? {
            return Ok(epoch);
        }
        let _lock = Self::lock(&self.directory)?;
        self.generation()
    }

    pub fn get(&self, key: &str, now_ms: u64) -> Result<Option<PersistentRecord>> {
        self.get_many(once(key), now_ms)
            .map(|mut records| records.pop().flatten())
    }

    pub fn get_many<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a str>,
        now_ms: u64,
    ) -> Result<Vec<Option<PersistentRecord>>> {
        let mut keys = keys.into_iter().peekable();
        if keys.peek().is_none() {
            return Ok(Vec::new());
        }
        let _lock = Self::lock(&self.directory)?;
        let mut snapshot = self.snapshot();
        self.refresh(&mut snapshot)?;
        let data = &mut snapshot
            .as_mut()
            .ok_or_else(|| io::Error::other("cache snapshot initialization failed"))?
            .data;
        let mut results = Vec::with_capacity(keys.size_hint().0);
        for key in keys {
            if data
                .records
                .get(key)
                .is_some_and(|record| record.expires_at_ms <= now_ms)
            {
                data.records.remove(key);
            }
            results.push(data.records.get_mut(key).map(|record| {
                record.accessed_at_ms = now_ms;
                record.clone()
            }));
        }
        drop(snapshot);
        Ok(results)
    }

    /// Returns false when a clear operation invalidated the caller's generation.
    pub fn insert(
        &self,
        epoch: u64,
        key: String,
        record: PersistentRecord,
        now_ms: u64,
    ) -> Result<bool> {
        self.insert_many(epoch, [(key, record)], now_ms)
    }

    /// Commits a set of outcomes atomically in the caller's cache generation.
    pub fn insert_many(
        &self,
        epoch: u64,
        records: impl IntoIterator<Item = (String, PersistentRecord)>,
        now_ms: u64,
    ) -> Result<bool> {
        let mut records = records.into_iter().peekable();
        if records.peek().is_none() {
            return Ok(self.epoch()? == epoch);
        }
        let _lock = Self::lock(&self.directory)?;
        let mut cached = self.snapshot();
        self.refresh(&mut cached)?;
        let snapshot = cached
            .as_mut()
            .ok_or_else(|| io::Error::other("cache snapshot initialization failed"))?;
        if snapshot.data.epoch != epoch {
            drop(cached);
            return Ok(false);
        }
        let result = {
            snapshot.data.records.extend(records);
            self.advance_revision(&mut snapshot.data)?;
            self.evict(&mut snapshot.data, now_ms)?;
            self.write_committed(&snapshot.data)
        };
        match result {
            Ok(token) => snapshot.token = token,
            Err(error) => {
                *cached = None;
                return Err(error);
            }
        }
        drop(cached);
        Ok(true)
    }

    pub fn clear(&self) -> Result<u64> {
        let _lock = Self::lock(&self.directory)?;
        let epoch = self
            .generation()?
            .checked_add(1)
            .ok_or_else(|| io::Error::other("cache generation exhausted"))?;
        write_atomic(&self.directory, "generation.json", &epoch)?;
        let mut data = Envelope {
            epoch,
            ..Envelope::default()
        };
        let token = self.commit(&mut data)?;
        *self.snapshot() = Some(Snapshot { token, data });
        Ok(epoch)
    }

    fn lock(directory: &Path) -> Result<File> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join("cache.lock"))?;
        file.lock()?;
        Ok(file)
    }

    fn generation(&self) -> Result<u64> {
        let epoch = match read_bounded(&self.directory.join("generation.json"), 64) {
            Ok(bytes) => {
                if let Ok(epoch) = serde_json::from_slice::<u64>(&bytes) {
                    return Ok(epoch);
                }
                return self.reset(self.previous_epoch()?);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                match read_bounded(&self.directory.join("cache.json"), self.max_bytes) {
                    Ok(bytes) => match serde_json::from_slice::<Envelope>(&bytes) {
                        Ok(data) if data.schema == SCHEMA => data.epoch,
                        _ => return self.reset(None),
                    },
                    Err(error) if error.kind() == io::ErrorKind::NotFound => 0,
                    Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                        return self.reset(None);
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                return self.reset(self.previous_epoch()?);
            }
            Err(error) => return Err(error),
        };
        write_atomic(&self.directory, "generation.json", &epoch)?;
        Ok(epoch)
    }

    fn refresh(&self, snapshot: &mut Option<Snapshot>) -> Result<()> {
        let token = self.token()?;
        if snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.token == token)
        {
            return Ok(());
        }
        *snapshot = Some(self.read(token)?);
        Ok(())
    }

    fn read(&self, token: SnapshotToken) -> Result<Snapshot> {
        let data = match read_bounded(&self.directory.join("cache.json"), self.max_bytes) {
            Ok(bytes) => match serde_json::from_slice::<Envelope>(&bytes) {
                Ok(data) if data.schema == SCHEMA && data.epoch == token.epoch => data,
                Ok(data) if data.schema == SCHEMA => Envelope {
                    epoch: token.epoch,
                    ..Envelope::default()
                },
                _ => return self.recover(),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => Envelope {
                epoch: token.epoch,
                ..Envelope::default()
            },
            Err(error) if error.kind() == io::ErrorKind::InvalidData => return self.recover(),
            Err(error) => return Err(error),
        };
        Ok(Snapshot { token, data })
    }

    fn recover(&self) -> Result<Snapshot> {
        let epoch = self.reset(self.previous_epoch()?)?;
        let token = self.token()?;
        Ok(Snapshot {
            token,
            data: Envelope {
                epoch,
                revision: token.revision,
                ..Envelope::default()
            },
        })
    }

    fn reset(&self, previous: Option<u64>) -> Result<u64> {
        let epoch = next_counter(previous)?;
        write_atomic(&self.directory, "generation.json", &epoch)?;
        let mut data = Envelope {
            epoch,
            ..Envelope::default()
        };
        self.commit(&mut data)?;
        Ok(epoch)
    }

    fn previous_epoch(&self) -> Result<Option<u64>> {
        let generation = read_number(&self.directory.join("generation.json"))?;
        if generation.is_some() {
            return Ok(generation);
        }
        Ok(
            read_optional_bounded(&self.directory.join("cache.json"), self.max_bytes)?
                .and_then(|bytes| serde_json::from_slice::<Envelope>(&bytes).ok())
                .filter(|data| data.schema == SCHEMA)
                .map(|data| data.epoch),
        )
    }

    fn token(&self) -> Result<SnapshotToken> {
        Ok(SnapshotToken {
            epoch: self.generation()?,
            revision: read_number(&self.directory.join("revision.json"))?.unwrap_or(0),
            cache: file_stamp(&self.directory.join("cache.json"))?,
        })
    }

    fn commit(&self, data: &mut Envelope) -> Result<SnapshotToken> {
        self.advance_revision(data)?;
        self.write_committed(data)
    }

    fn advance_revision(&self, data: &mut Envelope) -> Result<()> {
        let marker = read_number(&self.directory.join("revision.json"))?;
        data.revision = next_counter(Some(data.revision.max(marker.unwrap_or(0))))?;
        Ok(())
    }

    fn write_committed(&self, data: &Envelope) -> Result<SnapshotToken> {
        write_atomic(&self.directory, "cache.json", data)?;
        write_atomic(&self.directory, "revision.json", &data.revision)?;
        Ok(SnapshotToken {
            epoch: data.epoch,
            revision: data.revision,
            cache: file_stamp(&self.directory.join("cache.json"))?,
        })
    }

    fn evict(&self, data: &mut Envelope, now_ms: u64) -> Result<()> {
        data.records
            .retain(|_, record| record.expires_at_ms > now_ms);
        let mut encoded_bytes = serde_json::to_vec(data)?.len();
        let mut candidates = data
            .records
            .iter()
            .map(|(key, record)| {
                Ok((
                    key.clone(),
                    record.expires_at_ms > now_ms,
                    record.accessed_at_ms,
                    serde_json::to_vec(key)?.len() + 1 + serde_json::to_vec(record)?.len(),
                ))
            })
            .collect::<std::result::Result<Vec<_>, serde_json::Error>>()?;
        candidates.sort_by_key(|(_, fresh, accessed_at_ms, _)| (*fresh, *accessed_at_ms));
        let mut candidates = candidates.into_iter();
        while data.records.len() > self.max_records || encoded_bytes > self.max_bytes {
            let Some((key, _, _, entry_bytes)) = candidates.next() else {
                break;
            };
            let previous_len = data.records.len();
            data.records.remove(&key);
            encoded_bytes =
                encoded_bytes.saturating_sub(entry_bytes + usize::from(previous_len > 1));
        }
        Ok(())
    }

    fn snapshot(&self) -> MutexGuard<'_, Option<Snapshot>> {
        self.snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl std::fmt::Debug for PersistentCache {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PersistentCache")
            .field("directory", &self.directory)
            .field("max_records", &self.max_records)
            .field("max_bytes", &self.max_bytes)
            .finish_non_exhaustive()
    }
}

fn installation_secret(directory: &Path) -> Result<InstallationSecret> {
    let path = directory.join("identity.key");
    match read_bounded(&path, 32) {
        Ok(bytes) if bytes.len() == 32 => Ok(InstallationSecret {
            bytes: bytes
                .try_into()
                .map_err(|_| io::Error::other("invalid cache identity key"))?,
            replaced: false,
        }),
        Ok(_) => generate_installation_secret(directory, true),
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            generate_installation_secret(directory, true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let replaced =
                directory.join("generation.json").exists() || directory.join("cache.json").exists();
            generate_installation_secret(directory, replaced)
        }
        Err(error) => Err(error),
    }
}

fn generate_installation_secret(directory: &Path, replaced: bool) -> Result<InstallationSecret> {
    use ring::rand::SecureRandom;
    let mut secret = [0; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut secret)
        .map_err(|_| io::Error::other("cache identity generation failed"))?;
    write_atomic_bytes(directory, "identity.key", &secret, Some(0o600))?;
    Ok(InstallationSecret {
        bytes: secret,
        replaced,
    })
}

fn next_counter(previous: Option<u64>) -> Result<u64> {
    let previous = previous.unwrap_or(0);
    let next = previous
        .checked_add(1)
        .ok_or_else(|| io::Error::other("cache generation exhausted"))?;
    Ok(next.max(recovery_epoch()))
}

fn recovery_epoch() -> u64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(1, |duration| {
            duration.as_micros().try_into().unwrap_or(u64::MAX / 2)
        })
}

fn read_number(path: &Path) -> Result<Option<u64>> {
    Ok(read_optional_bounded(path, 64)?.and_then(|bytes| serde_json::from_slice(&bytes).ok()))
}

fn read_optional_bounded(path: &Path, limit: usize) -> Result<Option<Vec<u8>>> {
    match read_bounded(path, limit) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::InvalidData
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = File::open(path)?;
    let limit = u64::try_from(limit).unwrap_or(u64::MAX);
    let oversized = || io::Error::new(io::ErrorKind::InvalidData, "cache file exceeds size limit");
    if file.metadata()?.len() > limit {
        return Err(oversized());
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(oversized());
    }
    Ok(bytes)
}

fn file_stamp(path: &Path) -> Result<Option<FileStamp>> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(FileStamp {
            length: metadata.len(),
            modified: metadata.modified().ok(),
        })),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn write_atomic(directory: &Path, name: &str, data: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec(data)?;
    write_atomic_bytes(directory, name, &bytes, None)
}

fn write_atomic_bytes(directory: &Path, name: &str, bytes: &[u8], mode: Option<u32>) -> Result<()> {
    use std::io::Write;
    let temporary = directory.join(format!("{name}.pending"));
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(mode);
    }
    #[cfg(not(unix))]
    let _ = mode;
    let mut file = options.open(&temporary)?;
    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(mode))?;
    }
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, directory.join(name))?;
    #[cfg(unix)]
    File::open(directory)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

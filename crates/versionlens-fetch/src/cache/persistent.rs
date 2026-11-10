use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Result};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;

/// One cached outcome with the Unix-millisecond timestamps that govern its freshness and eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentRecord {
    /// The cached JSON payload.
    pub value: serde_json::Value,
    /// When the last attempt to fill this record was made.
    pub attempted_at_ms: u64,
    /// When the last attempt succeeded, or `None` if none has.
    pub succeeded_at_ms: Option<u64>,
    /// When the record stops being served and may be evicted.
    pub expires_at_ms: u64,
    /// The earliest time to retry after a failed attempt, or `None` when no retry is scheduled.
    pub retry_at_ms: Option<u64>,
    /// When the record was last read, which orders eviction.
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
    /// Opens the cache stored in `directory`, creating it when absent.
    ///
    /// # Errors
    /// Returns an I/O error when the directory, lock or installation secret cannot be created or read.
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

    /// Caps the cache at `max_records` records and `max_bytes` encoded bytes.
    #[must_use]
    pub fn with_limits(mut self, max_records: usize, max_bytes: usize) -> Self {
        self.max_records = max_records;
        self.max_bytes = max_bytes;
        self
    }

    /// A keyed HMAC-SHA256 digest of the length-prefixed `components`, as lowercase hex.
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

    /// The current cache generation, which [`PersistentCache::clear`] advances.
    ///
    /// # Errors
    /// Returns an I/O error when the generation cannot be read or written.
    pub fn epoch(&self) -> Result<u64> {
        if let Some(epoch) = read_number(&self.directory.join("generation.json"))? {
            return Ok(epoch);
        }
        let _lock = Self::lock(&self.directory)?;
        self.generation()
    }

    /// The unexpired record for `key`, marking it accessed at `now_ms`; expired records are removed.
    ///
    /// # Errors
    /// Returns an I/O error when the cache cannot be locked or read.
    pub fn get(&self, key: &str, now_ms: u64) -> Result<Option<PersistentRecord>> {
        self.get_many(once(key), now_ms)
            .map(|mut records| records.pop().flatten())
    }

    /// Looks up each of `keys` as [`PersistentCache::get`] does, in one pass and in the same order.
    ///
    /// # Errors
    /// Returns an I/O error when the cache cannot be locked or read.
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
    /// Stores `record` under `key` in generation `epoch`, returning false if the generation has since been cleared.
    ///
    /// # Errors
    /// Returns an I/O error when the cache cannot be locked, read or written.
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
    /// Stores every record in `records` in generation `epoch` as one write, returning false if the generation has since been cleared.
    ///
    /// # Errors
    /// Returns an I/O error when the cache cannot be locked, read or written.
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

    /// Removes every record and starts a new generation, which it returns.
    ///
    /// # Errors
    /// Returns an I/O error when the cache cannot be locked or written, or when the generation counter is exhausted.
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
#[expect(
    clippy::panic_in_result_fn,
    reason = "tests return io::Result so setup failures propagate with ?, and assert on the outcome"
)]
mod tests {
    use super::*;

    fn record(expires_at_ms: u64) -> PersistentRecord {
        PersistentRecord {
            value: serde_json::json!({"version":"2.0.0"}),
            attempted_at_ms: 1,
            succeeded_at_ms: Some(1),
            expires_at_ms,
            retry_at_ms: None,
            accessed_at_ms: 1,
        }
    }

    fn seed_cache(directory: &Path) -> Result<u64> {
        let cache = PersistentCache::open(directory)?;
        let epoch = cache.epoch()?;
        assert!(cache.insert(epoch, "stored".into(), record(100), 1)?);
        Ok(epoch)
    }

    fn empty_cache() -> Result<(PathBuf, PersistentCache, u64)> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let cache = PersistentCache::open(&directory)?;
        let epoch = cache.epoch()?;
        Ok((directory, cache, epoch))
    }

    #[test]
    fn batch_commit_survives_restart_and_rejects_invalidated_generation() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        assert!(cache.insert_many(
            epoch,
            (0..300).map(|index| (index.to_string(), record(100))),
            1,
        )?);
        let restarted = PersistentCache::open(&directory)?;
        for index in 0..300 {
            assert!(restarted.get(&index.to_string(), 2)?.is_some());
        }
        restarted.clear()?;
        assert!(!cache.insert_many(epoch, [("late".into(), record(100))], 3)?);
        assert!(restarted.get("late", 4)?.is_none());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn batch_lookup_preserves_positions_expiry_and_cross_handle_clear() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        cache.insert_many(
            epoch,
            [("fresh".into(), record(100)), ("expired".into(), record(2))],
            1,
        )?;
        let restarted = PersistentCache::open(&directory)?;
        let keys = ["missing", "fresh", "expired", "fresh"];
        let records = restarted.get_many(keys, 3)?;
        assert_eq!(
            records.iter().map(Option::is_some).collect::<Vec<_>>(),
            vec![false, true, false, true]
        );
        assert_eq!(records[1].as_ref().unwrap().accessed_at_ms, 3);
        cache.clear()?;
        assert!(restarted.get_many(keys, 4)?.iter().all(Option::is_none));
        fs::remove_dir_all(directory)
    }

    #[test]
    fn committed_epoch_can_be_read_while_a_writer_holds_the_lock() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        let lock = PersistentCache::lock(&directory)?;
        let observed = std::thread::scope(|scope| {
            let (sender, receiver) = std::sync::mpsc::channel();
            scope.spawn(move || sender.send(cache.epoch()).unwrap());
            let observed = receiver.recv_timeout(std::time::Duration::from_secs(1));
            drop(lock);
            observed
        });
        assert_eq!(
            observed.expect("epoch read waited for cache writer")?,
            epoch
        );
        fs::remove_dir_all(directory)
    }

    #[test]
    fn restart_preserves_records_and_clear_invalidates_writers() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        assert!(cache.insert(epoch, "first".into(), record(100), 1)?);
        let second = PersistentCache::open(&directory)?;
        assert_eq!(
            second.get("first", 2)?.map(|entry| entry.expires_at_ms),
            Some(100)
        );
        second.clear()?;
        assert!(!cache.insert(epoch, "late".into(), record(100), 2)?);
        assert!(cache.get("first", 3)?.is_none());
        assert!(cache.get("late", 3)?.is_none());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn eviction_prefers_expired_records() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let cache = PersistentCache::open(&directory)?.with_limits(2, 4096);
        let epoch = cache.epoch()?;
        for (key, expiry) in [("fresh", 100), ("expired", 2), ("new", 100)] {
            assert!(cache.insert(epoch, key.into(), record(expiry), 3)?);
        }
        assert!(cache.get("expired", 4)?.is_none());
        assert!(cache.get("fresh", 4)?.is_some());
        assert!(cache.get("new", 4)?.is_some());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn byte_limit_is_preserved_across_writes_and_restart() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let max_bytes = 700;
        let cache = PersistentCache::open(&directory)?.with_limits(10, max_bytes);
        let epoch = cache.epoch()?;
        for key in ["first", "second", "third"] {
            let mut value = record(100);
            value.value = serde_json::json!({"body": "x".repeat(256), "key": key});
            assert!(cache.insert(epoch, key.into(), value, 1)?);
            assert!(fs::metadata(directory.join("cache.json"))?.len() <= max_bytes as u64);
        }

        let restarted = PersistentCache::open(&directory)?.with_limits(10, max_bytes);
        assert!(restarted.get("first", 2)?.is_none());
        assert!(restarted.get("second", 2)?.is_none());
        assert!(restarted.get("third", 2)?.is_some());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn corrupted_data_recovers_with_stable_generation() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let cache = PersistentCache::open(&directory)?;
        fs::write(directory.join("cache.json"), "{")?;
        assert!(cache.get("missing", 1)?.is_none());
        let epoch = cache.epoch()?;
        assert_ne!(epoch, 0);
        assert_eq!(epoch, cache.epoch()?);
        assert!(cache.insert(epoch, "recovered".into(), record(100), 1)?);
        assert!(cache.get("recovered", 2)?.is_some());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn missing_generation_is_restored_from_the_committed_cache() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let epoch = seed_cache(&directory)?;
        fs::remove_file(directory.join("generation.json"))?;

        let restarted = PersistentCache::open(&directory)?;
        assert_eq!(restarted.epoch()?, epoch);
        assert!(restarted.get("stored", 2)?.is_some());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn corrupted_generation_invalidates_old_writers_and_recovers_across_restart() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let stale = PersistentCache::open(&directory)?;
        let old_epoch = stale.epoch()?;
        assert!(stale.insert(old_epoch, "old".into(), record(100), 1)?);
        fs::write(directory.join("generation.json"), "{")?;

        let recovered = PersistentCache::open(&directory)?;
        let new_epoch = recovered.epoch()?;
        assert_ne!(new_epoch, old_epoch);
        assert!(!stale.insert(old_epoch, "late".into(), record(100), 2)?);
        assert!(recovered.get("old", 2)?.is_none());
        assert!(recovered.insert(new_epoch, "new".into(), record(100), 2)?);

        let restarted = PersistentCache::open(&directory)?;
        assert_eq!(restarted.epoch()?, new_epoch);
        assert!(restarted.get("new", 3)?.is_some());
        fs::remove_dir_all(directory)
    }

    #[test]
    fn identity_loss_rotates_partitions_and_invalidates_persisted_records() -> Result<()> {
        for invalid in [false, true] {
            let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
            let stale = PersistentCache::open(&directory)?;
            let old_epoch = stale.epoch()?;
            let old_key = stale.partition_key(&[b"registry", b"credential"]);
            assert!(stale.insert(old_epoch, old_key.clone(), record(100), 1)?);
            if invalid {
                fs::write(directory.join("identity.key"), "invalid")?;
            } else {
                fs::remove_file(directory.join("identity.key"))?;
            }

            let recovered = PersistentCache::open(&directory)?;
            let new_epoch = recovered.epoch()?;
            let new_key = recovered.partition_key(&[b"registry", b"credential"]);
            assert_ne!(old_key, new_key);
            assert_ne!(old_epoch, new_epoch);
            assert!(recovered.get(&old_key, 2)?.is_none());
            assert!(!stale.insert(old_epoch, "late".into(), record(100), 2)?);

            let restarted = PersistentCache::open(&directory)?;
            assert_eq!(restarted.epoch()?, new_epoch);
            assert_eq!(
                restarted.partition_key(&[b"registry", b"credential"]),
                new_key
            );
            assert_eq!(fs::read(directory.join("identity.key"))?.len(), 32);
            fs::remove_dir_all(directory)?;
        }
        Ok(())
    }

    #[test]
    fn warm_lookup_updates_recency_without_rewriting_the_cache() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        seed_cache(&directory)?;

        let restarted = PersistentCache::open(&directory)?;
        let before = fs::read(directory.join("cache.json"))?;
        assert_eq!(restarted.get("stored", 2)?.unwrap().accessed_at_ms, 2);
        assert_eq!(restarted.get("stored", 3)?.unwrap().accessed_at_ms, 3);
        assert_eq!(fs::read(directory.join("cache.json"))?, before);
        fs::remove_dir_all(directory)
    }

    #[test]
    fn oversized_cache_files_recover_without_restoring_old_results() -> Result<()> {
        for (name, missing_generation) in [
            ("cache.json", false),
            ("cache.json", true),
            ("generation.json", false),
            ("identity.key", false),
        ] {
            let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
            let old_epoch = seed_cache(&directory)?;
            if missing_generation {
                fs::remove_file(directory.join("generation.json"))?;
            }
            File::create(directory.join(name))?.set_len(512 * 1024 * 1024)?;
            let cache = PersistentCache::open(&directory)?.with_limits(10, 4096);
            let epoch = cache.epoch()?;
            assert!(cache.get("stored", 2)?.is_none());
            let recovered_epoch = cache.epoch()?;
            assert_ne!(recovered_epoch, old_epoch);
            assert!(recovered_epoch >= epoch);
            assert!(!cache.insert(old_epoch, "late".into(), record(100), 2)?);
            assert!(cache.insert(recovered_epoch, "fresh".into(), record(100), 2)?);
            assert!(cache.get("fresh", 3)?.is_some());
            fs::remove_dir_all(directory)?;
        }
        Ok(())
    }

    #[test]
    fn expired_records_remain_unavailable_after_restart() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        assert!(cache.insert(epoch, "expired".into(), record(2), 1)?);
        assert!(
            PersistentCache::open(&directory)?
                .get("expired", 2)?
                .is_none()
        );
        assert!(
            PersistentCache::open(&directory)?
                .get("expired", 3)?
                .is_none()
        );
        fs::remove_dir_all(directory)
    }

    #[test]
    fn concurrent_handles_preserve_each_successful_write() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        std::thread::scope(|scope| -> Result<()> {
            let mut workers = Vec::new();
            for index in 0..8 {
                let directory = &directory;
                workers.push(scope.spawn(move || {
                    PersistentCache::open(directory)?.insert(
                        epoch,
                        index.to_string(),
                        record(100),
                        1,
                    )
                }));
            }
            for worker in workers {
                let written = worker
                    .join()
                    .map_err(|_| io::Error::other("cache worker panicked"))??;
                assert!(written);
            }
            Ok(())
        })?;
        for index in 0..8 {
            assert!(cache.get(&index.to_string(), 2)?.is_some());
        }
        fs::remove_dir_all(directory)
    }

    #[test]
    fn persistent_partitions_isolate_authentication_and_component_boundaries() -> Result<()> {
        let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
        let cache = PersistentCache::open(&directory)?;
        let first = cache.partition_key(&[b"registry", b"credential-a"]);
        let restarted = PersistentCache::open(&directory)?;
        assert_eq!(
            first,
            restarted.partition_key(&[b"registry", b"credential-a"])
        );
        assert_ne!(
            first,
            restarted.partition_key(&[b"registry", b"credential-b"])
        );
        assert_ne!(
            cache.partition_key(&[b"ab", b"c"]),
            cache.partition_key(&[b"a", b"bc"])
        );
        assert_eq!(first.len(), 64);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        fs::remove_dir_all(directory)
    }

    #[test]
    fn clear_racing_a_write_across_processes_leaves_no_stale_record() -> Result<()> {
        let (directory, cache, epoch) = empty_cache()?;
        assert!(cache.insert(epoch, "before".into(), record(100), 1)?);

        let mut child = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "cache::persistent::tests::persistent_cache_subprocess",
                "--nocapture",
            ])
            .env("VERSIONLENS_CACHE_HELPER_DIRECTORY", &directory)
            .spawn()?;
        let _ = cache.insert(epoch, "racing".into(), record(100), 2)?;
        let status = child.wait()?;
        assert!(status.success(), "cache subprocess failed: {status}");

        assert!(cache.get("before", 3)?.is_none());
        assert!(cache.get("racing", 3)?.is_none());
        assert!(!cache.insert(epoch, "late".into(), record(100), 3)?);
        fs::remove_dir_all(directory)
    }

    #[test]
    fn persistent_cache_subprocess() -> Result<()> {
        let Some(directory) = std::env::var_os("VERSIONLENS_CACHE_HELPER_DIRECTORY") else {
            return Ok(());
        };
        PersistentCache::open(directory)?.clear()?;
        Ok(())
    }

    #[cfg(unix)]
    mod read_only {
        use std::os::unix::fs::PermissionsExt;

        use super::*;

        /// Runs `check` on a fresh directory made read-only, and restores it.
        fn with_read_only_directory(
            seed: bool,
            check: impl FnOnce(&Path) -> Result<()>,
        ) -> Result<()> {
            let directory = versionlens_test_support::temporary_directory("versionlens-cache-ro")?;
            if seed {
                seed_cache(&directory)?;
            }
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o555))?;
            let result = check(&directory);
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))?;
            fs::remove_dir_all(&directory)?;
            result
        }

        #[test]
        fn a_cache_folder_that_cannot_be_created_fails_to_open() -> Result<()> {
            with_read_only_directory(false, |parent| {
                let error = PersistentCache::open(parent.join("cache")).unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
                assert!(!parent.join("cache").exists());
                Ok(())
            })
        }

        #[test]
        fn a_read_only_cache_still_serves_stored_records_and_rejects_writes() -> Result<()> {
            with_read_only_directory(true, |directory| {
                let cache = PersistentCache::open(directory)?;
                let epoch = cache.epoch()?;
                assert!(cache.get("stored", 1)?.is_some());
                assert!(cache.insert(epoch, "new".into(), record(100), 1).is_err());
                Ok(())
            })
        }
    }
}

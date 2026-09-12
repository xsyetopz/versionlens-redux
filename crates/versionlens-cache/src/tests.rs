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

fn seed_cache(directory: &Path) -> io::Result<u64> {
    let cache = PersistentCache::open(directory)?;
    let epoch = cache.epoch()?;
    assert!(cache.insert(epoch, "stored".into(), record(100), 1)?);
    Ok(epoch)
}

fn empty_cache() -> io::Result<(std::path::PathBuf, PersistentCache, u64)> {
    let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
    let cache = PersistentCache::open(&directory)?;
    let epoch = cache.epoch()?;
    Ok((directory, cache, epoch))
}

#[test]
fn batch_commit_survives_restart_and_rejects_invalidated_generation() -> io::Result<()> {
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
fn batch_lookup_preserves_positions_expiry_and_cross_handle_clear() -> io::Result<()> {
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
fn committed_epoch_can_be_read_while_a_writer_holds_the_lock() -> io::Result<()> {
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
fn restart_preserves_records_and_clear_invalidates_writers() -> io::Result<()> {
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
fn eviction_prefers_expired_records() -> io::Result<()> {
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
fn byte_limit_is_preserved_across_writes_and_restart() -> io::Result<()> {
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
fn corrupted_data_recovers_with_stable_generation() -> io::Result<()> {
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
fn missing_generation_is_restored_from_the_committed_cache() -> io::Result<()> {
    let directory = versionlens_test_support::temporary_directory("versionlens-cache")?;
    let epoch = seed_cache(&directory)?;
    fs::remove_file(directory.join("generation.json"))?;

    let restarted = PersistentCache::open(&directory)?;
    assert_eq!(restarted.epoch()?, epoch);
    assert!(restarted.get("stored", 2)?.is_some());
    fs::remove_dir_all(directory)
}

#[test]
fn corrupted_generation_invalidates_old_writers_and_recovers_across_restart() -> io::Result<()> {
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
fn identity_loss_rotates_partitions_and_invalidates_persisted_records() -> io::Result<()> {
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
fn warm_lookup_updates_recency_without_rewriting_the_cache() -> io::Result<()> {
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
fn oversized_cache_files_recover_without_restoring_old_results() -> io::Result<()> {
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
fn expired_records_remain_unavailable_after_restart() -> io::Result<()> {
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
fn concurrent_handles_preserve_each_successful_write() -> io::Result<()> {
    let (directory, cache, epoch) = empty_cache()?;
    std::thread::scope(|scope| -> io::Result<()> {
        let mut workers = Vec::new();
        for index in 0..8 {
            let directory = &directory;
            workers.push(scope.spawn(move || {
                PersistentCache::open(directory)?.insert(epoch, index.to_string(), record(100), 1)
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
fn persistent_partitions_isolate_authentication_and_component_boundaries() -> io::Result<()> {
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
fn clear_racing_a_write_across_processes_leaves_no_stale_record() -> io::Result<()> {
    let (directory, cache, epoch) = empty_cache()?;
    assert!(cache.insert(epoch, "before".into(), record(100), 1)?);

    let mut child = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "persistent::tests::persistent_cache_subprocess",
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
fn persistent_cache_subprocess() -> io::Result<()> {
    let Some(directory) = std::env::var_os("VERSIONLENS_CACHE_HELPER_DIRECTORY") else {
        return Ok(());
    };
    PersistentCache::open(directory)?.clear()?;
    Ok(())
}

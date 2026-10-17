use std::env::var_os;
use std::io;
use std::path::PathBuf;

pub fn application_cache_directory() -> io::Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = var_os("HOME").map(|home| PathBuf::from(home).join("Library/Caches"));
    #[cfg(windows)]
    let base = var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(any(target_os = "macos", windows)))]
    let base = var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| var_os("HOME").map(|home| PathBuf::from(home).join(".cache")));
    base.filter(|path| path.is_absolute())
        .map(|path| path.join("versionlens-redux"))
        .ok_or_else(|| io::Error::other("application cache directory is unavailable"))
}

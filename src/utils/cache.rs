use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use tracing::{debug, warn};

const CACHE_TTL_SECS: u64 = 60; // 1 minute

/// Get the cache directory path
fn get_cache_dir() -> PathBuf {
    std::env::temp_dir().join("aipriceaction-proxy-cache")
}

/// Initialize cache directory
pub fn init_cache_dir() -> io::Result<()> {
    let cache_dir = get_cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
        debug!(?cache_dir, "Created cache directory");
    }
    Ok(())
}

/// Convert a path to a safe cache filename
fn path_to_cache_filename(path: &str) -> String {
    // Replace path separators and special characters with underscores
    path.replace('/', "_").replace('\\', "_").replace("..", "_")
}

/// Get the full path to a cached file
fn get_cache_file_path(path: &str) -> PathBuf {
    get_cache_dir().join(path_to_cache_filename(path))
}

/// Check if a cached file exists and is still valid (within TTL)
pub fn is_cache_valid(path: &str) -> bool {
    let cache_file = get_cache_file_path(path);

    if !cache_file.exists() {
        debug!(?path, "Cache miss: file doesn't exist");
        return false;
    }

    match fs::metadata(&cache_file) {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(modified_time) => {
                    match SystemTime::now().duration_since(modified_time) {
                        Ok(age) => {
                            let is_valid = age < Duration::from_secs(CACHE_TTL_SECS);
                            if is_valid {
                                debug!(?path, age_secs = age.as_secs(), "Cache hit: file is valid");
                            } else {
                                debug!(?path, age_secs = age.as_secs(), ttl_secs = CACHE_TTL_SECS, "Cache expired");
                            }
                            is_valid
                        }
                        Err(e) => {
                            warn!(?path, ?e, "Failed to calculate file age");
                            false
                        }
                    }
                }
                Err(e) => {
                    warn!(?path, ?e, "Failed to get file modified time");
                    false
                }
            }
        }
        Err(e) => {
            warn!(?path, ?e, "Failed to get file metadata");
            false
        }
    }
}

/// Read cached file content
pub fn read_cache(path: &str) -> io::Result<Vec<u8>> {
    let cache_file = get_cache_file_path(path);
    debug!(?path, ?cache_file, "Reading from cache");
    fs::read(&cache_file)
}

/// Write content to cache
pub fn write_cache(path: &str, content: &[u8]) -> io::Result<()> {
    // Ensure cache directory exists
    init_cache_dir()?;

    let cache_file = get_cache_file_path(path);
    debug!(?path, ?cache_file, content_size = content.len(), "Writing to cache");
    fs::write(&cache_file, content)
}

/// Clear cache for a specific path
pub fn clear_cache(path: &str) -> io::Result<()> {
    let cache_file = get_cache_file_path(path);
    if cache_file.exists() {
        debug!(?path, ?cache_file, "Clearing cache");
        fs::remove_file(&cache_file)?;
    }
    Ok(())
}

/// Clear all cache files older than TTL
pub fn cleanup_old_cache() -> io::Result<()> {
    let cache_dir = get_cache_dir();
    if !cache_dir.exists() {
        return Ok(());
    }

    let now = SystemTime::now();
    let mut removed_count = 0;

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > Duration::from_secs(CACHE_TTL_SECS) {
                        if fs::remove_file(&path).is_ok() {
                            removed_count += 1;
                        }
                    }
                }
            }
        }
    }

    if removed_count > 0 {
        debug!(removed_count, "Cleaned up old cache files");
    }

    Ok(())
}

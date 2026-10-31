use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const CACHE_HEADER: &str = "corrplot-ng-cache-v1";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Compute a stable local cache fingerprint over all source files and render
/// options. This is an invalidation key, not a security checksum.
pub fn fingerprint(
    paths: &[PathBuf],
    options: &[&str],
) -> Result<String, String> {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x84222325cbf29ce4_u64;
    hash_part(&mut first, &mut second, CACHE_HEADER.as_bytes());
    for option in options {
        hash_part(&mut first, &mut second, option.as_bytes());
    }
    for path in paths {
        let canonical = fs::canonicalize(path)
            .map_err(|error| format!("cannot resolve cache input '{}': {error}", path.display()))?;
        let bytes = fs::read(&canonical)
            .map_err(|error| format!("cannot read cache input '{}': {error}", canonical.display()))?;
        hash_part(&mut first, &mut second, canonical.to_string_lossy().as_bytes());
        hash_part(&mut first, &mut second, &bytes);
    }
    Ok(format!("{first:016x}{second:016x}"))
}

fn hash_part(first: &mut u64, second: &mut u64, bytes: &[u8]) {
    for byte in bytes.iter().copied().chain(std::iter::once(0xff)) {
        *first ^= byte as u64;
        *first = first.wrapping_mul(0x100000001b3);
        *second ^= (byte as u64).rotate_left(1);
        *second = second.wrapping_mul(0x100000001b3);
    }
}

pub fn cache_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_os_string();
    name.push(".corrplot-cache");
    PathBuf::from(name)
}

pub fn is_fresh(output: &Path, fingerprint: &str) -> bool {
    let Ok(output_bytes) = fs::read(output) else {
        return false;
    };
    let expected = format!(
        "{CACHE_HEADER}\n{fingerprint}\n{}\n",
        content_fingerprint(&output_bytes),
    );
    fs::read_to_string(cache_path(output))
        .map(|actual| actual == expected)
        .unwrap_or(false)
}

pub fn store_fingerprint(output: &Path, fingerprint: &str) -> Result<(), String> {
    let output_bytes = fs::read(output)
        .map_err(|error| format!("cannot read rendered output '{}': {error}", output.display()))?;
    let value = format!(
        "{CACHE_HEADER}\n{fingerprint}\n{}\n",
        content_fingerprint(&output_bytes),
    );
    write_atomic(&cache_path(output), value.as_bytes())
        .map(|_| ())
        .map_err(|error| format!("cannot write cache metadata for '{}': {error}", output.display()))
}

fn content_fingerprint(bytes: &[u8]) -> String {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x84222325cbf29ce4_u64;
    hash_part(&mut first, &mut second, bytes);
    format!("{first:016x}{second:016x}")
}

/// Write a complete file beside its destination and replace the destination
/// only after the new contents have been flushed successfully.
pub fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<bool> {
    if let Ok(existing) = fs::read(path) {
        if existing == contents {
            return Ok(false);
        }
    }

    let parent = path.parent().filter(|value| !value.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{file_name}.tmp-{}-{sequence}", std::process::id()));

    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        replace_file(&temporary, path)?;
        Ok(true)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let destination: Vec<u16> = destination.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
    let result = unsafe {
        MoveFileExW(source.as_ptr(), destination.as_ptr(), MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("corrplot-cache-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn fingerprint_changes_with_input_content_and_renderer_version() {
        let dir = temp_dir("fingerprint");
        let input = dir.join("matrix.csv");
        fs::write(&input, ",a,b\na,1,0.5\nb,0.5,1\n").unwrap();
        let paths = vec![input.clone()];
        let first = fingerprint(&paths, &["svg", "renderer-v1"]).unwrap();
        let same = fingerprint(&paths, &["svg", "renderer-v1"]).unwrap();
        let next_renderer = fingerprint(&paths, &["svg", "renderer-v2"]).unwrap();
        fs::write(&input, ",a,b\na,1,0.7\nb,0.7,1\n").unwrap();
        let next_input = fingerprint(&paths, &["svg", "renderer-v1"]).unwrap();
        assert_eq!(first, same);
        assert_ne!(first, next_renderer);
        assert_ne!(first, next_input);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn cache_requires_matching_metadata_and_existing_output() {
        let dir = temp_dir("fresh");
        let output = dir.join("figure.svg");
        assert!(!is_fresh(&output, "abc"));
        write_atomic(&output, b"<svg/>").unwrap();
        store_fingerprint(&output, "abc").unwrap();
        assert!(is_fresh(&output, "abc"));
        assert!(!is_fresh(&output, "different"));
        fs::remove_file(&output).unwrap();
        assert!(!is_fresh(&output, "abc"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn atomic_write_replaces_complete_contents_and_skips_identical_bytes() {
        let dir = temp_dir("atomic");
        let output = dir.join("figure.svg");
        assert!(write_atomic(&output, b"first").unwrap());
        assert!(!write_atomic(&output, b"first").unwrap());
        assert!(write_atomic(&output, b"second").unwrap());
        assert_eq!(fs::read(&output).unwrap(), b"second");
        assert_eq!(
            fs::read_dir(&dir).unwrap().count(),
            1,
            "temporary files must not remain after replacement",
        );
        fs::remove_dir_all(dir).ok();
    }
}

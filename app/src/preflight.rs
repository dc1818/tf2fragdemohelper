use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use sysinfo::Disks;

#[derive(Clone, Debug)]
pub struct DiskSpace {
    pub mount_point: PathBuf,
    pub available_bytes: u64,
}

/// Resolve free space for the filesystem that will contain `destination`.
/// The destination itself does not have to exist yet; its nearest existing
/// parent is used so pre-flight checks can run before creating output folders.
pub fn disk_space_for(destination: &Path) -> Result<DiskSpace> {
    let target = absolute_path(destination)?;
    let existing = nearest_existing_parent(&target)
        .with_context(|| format!("no existing parent was found for {}", destination.display()))?;
    let disks = Disks::new_with_refreshed_list();
    let target_text = existing.to_string_lossy().to_ascii_lowercase();
    let mut matches = disks
        .list()
        .iter()
        .filter(|disk| {
            existing.starts_with(disk.mount_point())
                || target_text.starts_with(&disk.mount_point().to_string_lossy().to_ascii_lowercase())
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|disk| disk.mount_point().components().count());
    let disk = matches.last().copied().with_context(|| {
        format!("could not determine free disk space for {}", destination.display())
    })?;
    Ok(DiskSpace {
        mount_point: disk.mount_point().to_path_buf(),
        available_bytes: disk.available_space(),
    })
}

pub fn require_disk_space(destination: &Path, required_bytes: u64, purpose: &str) -> Result<DiskSpace> {
    let disk = disk_space_for(destination)?;
    if disk.available_bytes < required_bytes {
        bail!(
            "not enough free space for {purpose} on {}: {} required (including safety headroom), but only {} is available",
            disk.mount_point.display(),
            format_bytes(required_bytes as f64),
            format_bytes(disk.available_bytes as f64),
        );
    }
    Ok(disk)
}

pub fn format_bytes(mut bytes: f64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut unit = 0usize;
    while bytes >= 1024.0 && unit + 1 < units.len() {
        bytes /= 1024.0;
        unit += 1;
    }
    if unit == 0 || bytes >= 100.0 {
        format!("{bytes:.0} {}", units[unit])
    } else {
        format!("{bytes:.1} {}", units[unit])
    }
}

pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = seconds % 3600 / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        bail!("choose an output location before starting");
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn nearest_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        if current.exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_and_duration_formatting_is_readable() {
        assert_eq!(format_bytes(1_073_741_824.0), "1.0 GB");
        assert_eq!(format_duration(3_661), "1h 1m 1s");
    }
}

//! Session cleanup utilities.

use std::path::Path;

/// Removes old screenshot sessions, keeping only the most recent `retain_count`.
///
/// Session directories are identified by their numeric (timestamp) names.
pub fn cleanup_old_sessions(screenshot_dir: &Path, retain_count: usize) {
    if retain_count == 0 {
        return;
    }

    let Ok(entries) = std::fs::read_dir(screenshot_dir) else {
        return;
    };

    let mut sessions: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.chars()
                .all(|c| c.is_ascii_digit())
                .then(|| (name, e.path()))
        })
        .collect();

    sessions.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, path) in sessions.into_iter().skip(retain_count) {
        let _ = std::fs::remove_dir_all(path);
    }
}

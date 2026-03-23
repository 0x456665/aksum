//! Auto-updater for entry point files.
//!
//! When generating new modules, controllers, or services, this module
//! handles inserting new code at marker comments (e.g. `// aksum:module_routes`)
//! in existing source files.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// Insert a line of code just before a marker comment in a file.
///
/// Finds the marker string in the file and inserts `new_line` on the
/// line above it, preserving the marker for future insertions.
///
/// # Arguments
/// * `file_path` - Path to the file to update
/// * `marker` - The marker comment to find (e.g. "// aksum:module_routes")
/// * `new_line` - The line of code to insert before the marker
pub fn insert_at_marker(file_path: &Path, marker: &str, new_line: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Check if the line already exists (idempotency)
    if content.contains(new_line.trim()) {
        println!("  ⏭ Already present in {}", file_path.display());
        return Ok(());
    }

    // Find the marker and insert before it
    if !content.contains(marker) {
        bail!(
            "Marker '{}' not found in {}. Was the file modified manually?",
            marker,
            file_path.display()
        );
    }

    let updated = content.replace(marker, &format!("{}\n{}", new_line, marker));

    fs::write(file_path, updated)
        .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

    println!("  ✔ Updated {}", file_path.display());
    Ok(())
}

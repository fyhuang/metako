use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use crate::save::types::{SaveError, SaveTarget};

/// Download a save target to the specified directory
pub fn download(target: &SaveTarget, dest_dir: &Path) -> Result<(), SaveError> {
    match target {
        SaveTarget::YtDlp { url } => download_ytdlp(url, dest_dir),
        SaveTarget::GalleryDl { url } => download_gallerydl(url, dest_dir),
    }
}

fn download_ytdlp(url: &str, dest_dir: &Path) -> Result<(), SaveError> {
    let output = Command::new("yt-dlp")
        .current_dir(dest_dir)
        .args([
            "--write-info-json",
            "--restrict-filenames",
            url,
        ])
        .output()?;

    if !output.status.success() {
        return Err(SaveError::CommandFailed {
            command: "yt-dlp".to_string(),
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(())
}

/// Download a gallery, or a single file (image) to the specified directory.
/// Also write an .info.json file to record information about the original link/URL.
///
/// If the target is a single file, the .info.json file is named after the file itself:
///     filename.jpg --> filename.info.json
/// If the target is a gallery (containing multiple files), one info.json file is created to
/// represent the gallery as a whole, and is simply named `info.json`.
fn download_gallerydl(url: &str, dest_dir: &Path) -> Result<(), SaveError> {
    // Snapshot existing files before download
    let existing_files: HashSet<_> = std::fs::read_dir(dest_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let output = Command::new("gallery-dl")
        .args([
            "--write-metadata",
            "-D",
            dest_dir.to_str().expect("dest_dir must be valid UTF-8"),
            "-f",
            "{filename}.{extension}",
            url,
        ])
        .output()?;

    if !output.status.success() {
        return Err(SaveError::CommandFailed {
            command: "gallery-dl".to_string(),
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    // Collect newly downloaded files (not in existing_files)
    let mut new_media_files = vec![];
    let mut new_json_files = vec![];

    for entry in std::fs::read_dir(dest_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip files that existed before the download
        if existing_files.contains(&path) {
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename.ends_with(".json") && !filename.ends_with(".info.json") {
                new_json_files.push(path);
            } else if path.is_file() {
                new_media_files.push(path);
            }
        }
    }

    // Apply appropriate renaming strategy based on number of newly downloaded media files
    if new_media_files.len() == 1 {
        // Single file: rename to {filename}.info.json
        rename_single_file_metadata(&new_json_files, dest_dir)?;
    } else if new_media_files.len() > 1 {
        // Gallery: keep first .json as info.json, delete others
        rename_gallery_metadata(&new_json_files, dest_dir)?;
    }

    Ok(())
}

/// Rename metadata file for a single-file download
/// Renames {filename}.{ext}.json → {filename}.info.json
fn rename_single_file_metadata(json_files: &[std::path::PathBuf], dest_dir: &Path) -> Result<(), SaveError> {
    if json_files.is_empty() {
        return Ok(()); // No metadata to rename
    }

    // For single-file downloads, there should be exactly one .json file
    // Rename it to {base_name}.info.json
    for json_path in json_files {
        if let Some(filename) = json_path.file_name().and_then(|s| s.to_str()) {
            // Strip .json to get {filename}.{ext}
            let without_json = &filename[..filename.len() - 5]; // remove ".json"

            // Strip the file extension to get just {filename}
            let base_name = if let Some(dot_pos) = without_json.rfind('.') {
                &without_json[..dot_pos]
            } else {
                without_json
            };

            let new_filename = format!("{}.info.json", base_name);
            let new_path = dest_dir.join(new_filename);
            std::fs::rename(json_path, &new_path)?;
        }
    }

    Ok(())
}

/// Rename metadata files for a gallery download
/// Keeps the first .json file as info.json, deletes the rest
fn rename_gallery_metadata(json_files: &[std::path::PathBuf], dest_dir: &Path) -> Result<(), SaveError> {
    if json_files.is_empty() {
        return Ok(()); // No metadata to process
    }

    // Sort to ensure deterministic behavior (alphabetically)
    let mut sorted_json = json_files.to_vec();
    sorted_json.sort();

    // Rename first .json to info.json
    let info_path = dest_dir.join("info.json");
    std::fs::rename(&sorted_json[0], &info_path)?;

    // Delete the rest
    for json_file in &sorted_json[1..] {
        std::fs::remove_file(json_file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_download_ytdlp_integration() {
        // Use a short test video
        let temp_dir = tempfile::tempdir().unwrap();
        let target = SaveTarget::YtDlp {
            url: "https://www.youtube.com/watch?v=jNQXAC9IVRw".to_string(), // "Me at the zoo"
        };

        let result = download(&target, temp_dir.path());
        assert!(result.is_ok(), "Download failed: {:?}", result.err());

        // Check that files were created (exact filenames depend on yt-dlp output)
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!entries.is_empty(), "No files downloaded");

        // Should have at least a video file and an info.json
        let has_video = entries.iter().any(|e| {
            let path = e.path();
            path.extension()
                .and_then(|s| s.to_str())
                .map(|s| ["mp4", "webm", "mkv"].contains(&s))
                .unwrap_or(false)
        });
        let has_info_json = entries
            .iter()
            .any(|e| e.path().to_str().unwrap().ends_with(".info.json"));

        assert!(has_video, "No video file found");
        assert!(has_info_json, "No info.json found");
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_download_gallerydl_integration() {
        // Use an Imgur post (gallery-dl supports Imgur)
        let temp_dir = tempfile::tempdir().unwrap();
        let target = SaveTarget::GalleryDl {
            url: "https://imgur.com/gallery/brief-history-of-imgur-EBCuk".to_string(),
        };

        let result = download(&target, temp_dir.path());
        assert!(result.is_ok(), "Download failed: {:?}", result.err());

        // Check that files were created
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!entries.is_empty(), "No files downloaded");

        // Count media files and metadata files
        let media_files: Vec<_> = entries
            .iter()
            .filter(|e| {
                let path = e.path();
                path.extension()
                    .and_then(|s| s.to_str())
                    .map(|s| ["jpg", "jpeg", "png", "gif", "webp"].contains(&s))
                    .unwrap_or(false)
            })
            .collect();

        let info_json_files: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s == "info.json")
            })
            .collect();

        let other_json_files: Vec<_> = entries
            .iter()
            .filter(|e| {
                let path = e.path();
                let filename = path.file_name().and_then(|s| s.to_str());
                filename.is_some_and(|s| s.ends_with(".json") && s != "info.json")
            })
            .collect();

        // Verify gallery metadata behavior
        assert!(
            media_files.len() > 1,
            "Should have multiple media files for a gallery (found {})",
            media_files.len()
        );
        assert_eq!(
            info_json_files.len(),
            1,
            "Should have exactly one info.json file for gallery"
        );
        assert_eq!(
            other_json_files.len(),
            0,
            "Should have no other .json files (found {} files)",
            other_json_files.len()
        );
    }

    #[test]
    #[ignore]
    fn test_download_gallerydl_info_json_naming() {
        // Test that gallery-dl creates correctly named info.json files
        let temp_dir = tempfile::tempdir().unwrap();
        let target = SaveTarget::GalleryDl {
            url: "https://i.imgur.com/dEuFl5h.png".to_string(),
        };

        download(&target, temp_dir.path()).expect("download failed");

        // Check files created
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // Should have the image file
        let has_png = entries.iter().any(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("png")
        });
        assert!(has_png, "No PNG file found");

        // Should have info.json file (not .png.json)
        let info_json_files: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.ends_with(".info.json"))
            })
            .collect();

        assert_eq!(info_json_files.len(), 1, "Should have exactly one .info.json file");

        // Verify the naming pattern: {filename}.info.json (not {filename}.{ext}.info.json)
        let info_path = info_json_files[0].path();
        let info_filename = info_path.file_name().unwrap().to_str().unwrap();

        // Should be dEuFl5h.info.json, not dEuFl5h.png.info.json
        assert!(
            !info_filename.contains(".png.info.json"),
            "Info json should not include .png extension: {}",
            info_filename
        );
        assert!(
            info_filename.ends_with(".info.json"),
            "Info json should end with .info.json: {}",
            info_filename
        );
    }

    #[test]
    #[ignore]
    fn test_download_gallerydl_with_preexisting_files() {
        // Test that pre-existing files don't interfere with single-file vs gallery detection
        let temp_dir = tempfile::tempdir().unwrap();

        // Create some pre-existing files
        std::fs::write(temp_dir.path().join("existing1.jpg"), b"fake image 1").unwrap();
        std::fs::write(temp_dir.path().join("existing2.png"), b"fake image 2").unwrap();
        std::fs::write(temp_dir.path().join("existing.json"), b"{}").unwrap();

        // Download a single file
        let target = SaveTarget::GalleryDl {
            url: "https://i.imgur.com/dEuFl5h.png".to_string(),
        };

        download(&target, temp_dir.path()).expect("download failed");

        // Verify that only the newly downloaded file gets a .info.json (not the pre-existing files)
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let info_json_files: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.ends_with(".info.json"))
            })
            .collect();

        // Should have exactly one .info.json file (for the newly downloaded file)
        assert_eq!(
            info_json_files.len(),
            1,
            "Should have exactly one .info.json file despite pre-existing files"
        );

        // The .info.json should be for the newly downloaded file, not the pre-existing ones
        let info_path = info_json_files[0].path();
        let info_filename = info_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            !info_filename.starts_with("existing"),
            "Info json should not be for pre-existing files: {}",
            info_filename
        );

        // Pre-existing files should still exist and not have .info.json files
        assert!(temp_dir.path().join("existing1.jpg").exists());
        assert!(temp_dir.path().join("existing2.png").exists());
        assert!(temp_dir.path().join("existing.json").exists());
        assert!(!temp_dir.path().join("existing1.info.json").exists());
        assert!(!temp_dir.path().join("existing2.info.json").exists());
    }
}

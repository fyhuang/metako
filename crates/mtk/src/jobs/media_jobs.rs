use std::path::{Path, PathBuf};

use crate::{
    file_tree::{GeneratedFile, GeneratedTree},
    media::video::subtitle,
    media::video::{self, transcode::TranscodeProfile},
    preview,
};

pub fn generate_video_info_job_spec() -> impl crate::jobs::JobSpec {
    super::misc_jobs::UpdateGeneratedNotesJobSpec::new(
        "video_info",
        video::VIDEO_INFO_GROUP_NAME,
        |entry: &crate::Entry| crate::filetype::is_video(&entry.fs.file_path),
        |fs_entry: &crate::FsEntry| video::get_video_info(&fs_entry.file_path),
    )
}

////////////////////////////////
// PreviewJobSpec
////////////////////////////////
pub struct PreviewJobSpec;

/// Standalone function for the preview job logic
fn run_preview_job(
    media_path: &Path,
    preview_out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if crate::filetype::is_video(media_path) {
        if preview_out_path.exists() {
            return Ok(()); // Preview already exists, nothing to do
        }

        println!("PreviewJob: making preview for {:?}", media_path);
        crate::media::video::video_preview::make_preview_image(media_path, preview_out_path);
    } else if crate::filetype::is_image(media_path) {
        if preview_out_path.exists() {
            return Ok(()); // Preview already exists, nothing to do
        }

        println!("PreviewJob: making preview for {:?}", media_path);
        crate::media::image::make_preview_image(media_path, preview_out_path);
    } else {
        // Other filetypes not supported yet
        return Err(format!("Unsupported file type for preview: {:?}", media_path).into());
    }

    Ok(())
}

impl super::JobSpec for PreviewJobSpec {
    fn job_type(&self) -> &str {
        "preview"
    }

    fn create_job(
        &self,
        stash: &crate::Vault,
        entry: &crate::Entry,
    ) -> Result<Option<Box<crate::jobs::JobFn>>, Box<dyn std::error::Error>> {
        if crate::filetype::is_video(&entry.fs.file_path) {
        } else if crate::filetype::is_image(&entry.fs.file_path) {
        } else {
            return Ok(None);
        }

        if entry.fs.file_path.is_dir() {
            // TODO: skip directories for now (later, generate dir previews from the contents)
            return Ok(None);
        }

        let gen_tree = stash.new_generated_tree();
        let preview_path = preview::get_preview(entry.db.id, &gen_tree);
        if preview_path.exists() {
            // Preview already exists, don't create a new one
            return Ok(None);
        }

        let media_path = entry.fs.file_path.clone();
        Ok(Some(Box::new(move || {
            run_preview_job(&media_path, &preview_path)
        })))
    }
}

////////////////////////////////
// TranscodeChromeJobSpec
////////////////////////////////

/// Transcode video to something that Chrome can play.
pub struct TranscodeChromeJobSpec {
    // TODO: specify the target profile
}

impl crate::jobs::JobSpec for TranscodeChromeJobSpec {
    fn job_type(&self) -> &str {
        "transcode"
    }

    fn create_job(
        &self,
        stash: &crate::Vault,
        entry: &crate::Entry,
    ) -> Result<Option<Box<crate::jobs::JobFn>>, Box<dyn std::error::Error>> {
        if !is_transcode_needed_chrome(entry) {
            return Ok(None);
        }
        let video_path = entry.fs.file_path.clone();
        let entry_id = entry.db.id;
        let gen_tree = stash.new_generated_tree();
        Ok(Some(Box::new(move || {
            run_transcode_job(&video_path, entry_id, &gen_tree)
        })))
    }
}

/// Standalone function for the transcode job logic, for testability
pub fn run_transcode_job(
    video_path: &Path,
    entry_id: i64,
    gen_tree: &GeneratedTree,
) -> Result<(), Box<dyn std::error::Error>> {
    for profile in [TranscodeProfile::AV1_400K] {
        let gfile = GeneratedFile {
            entry_id,
            file_type: crate::file_tree::GeneratedFileType::AltFormat,
            metadata: "av1_400k".to_string(),
            extension: "webm".to_string(),
        };
        let gen_path = gen_tree.path_to_generated_file(&gfile);
        if !gen_path.exists() {
            println!(
                "TranscodeJob: transcoding {:?} ({}) to {:?}",
                video_path, entry_id, profile
            );
            video::transcode::transcode_alt_format(video_path, &gen_path, profile);
        }
    }
    Ok(())
}

/// Standalone function to check if a transcode job is needed
fn is_transcode_needed_chrome(entry: &crate::Entry) -> bool {
    if !crate::filetype::is_video(&entry.fs.file_path) {
        return false;
    }
    match crate::catalog::generated_notes::read::<video::VideoInfo>(
        &entry.db,
        video::VIDEO_INFO_GROUP_NAME,
    ) {
        Some(video_info) => !video::chrome_can_play(&video_info),
        None => {
            // Don't do anything if we don't have video info yet. We don't
            // want to over-transcode and waste CPU/disk.
            false
        }
    }
}

////////////////////////////////
// ConvertSubtitlesJobSpec
////////////////////////////////

/// Standalone function for the subtitle conversion job
fn run_convert_subtitles_job(
    to_convert: Vec<(subtitle::Subtitle, PathBuf)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (subtitle, dest_file) in to_convert {
        println!(
            "ConvertSubtitlesJob: converting {:?} -> {:?}",
            subtitle, &dest_file
        );
        subtitle::convert_to_vtt(&subtitle, &dest_file);
    }
    Ok(())
}

pub struct ConvertSubtitlesJobSpec;

impl crate::jobs::JobSpec for ConvertSubtitlesJobSpec {
    fn job_type(&self) -> &str {
        "subtitles"
    }
    fn create_job(
        &self,
        stash: &crate::Vault,
        entry: &crate::Entry,
    ) -> Result<Option<Box<crate::jobs::JobFn>>, Box<dyn std::error::Error>> {
        if !crate::filetype::is_video(&entry.fs.file_path) {
            return Ok(None);
        }

        let video_path = entry.fs.file_path.clone();
        let entry_id = entry.db.id;
        let gen_tree = stash.new_generated_tree();

        let potential_to_convert = subtitle::find_non_vtt_subtitles(&video_path);
        let mut to_convert = Vec::new();
        for (lang_country, subtitle) in potential_to_convert {
            let gfile = GeneratedFile {
                entry_id,
                file_type: crate::file_tree::GeneratedFileType::Subtitle,
                metadata: lang_country,
                extension: "vtt".to_string(),
            };
            let path = gen_tree.path_to_generated_file(&gfile);
            if path.exists() {
                // Already converted
                continue;
            }
            to_convert.push((subtitle, path));
        }

        if to_convert.is_empty() {
            // No subtitles to convert
            return Ok(None);
        }
        Ok(Some(Box::new(move || {
            run_convert_subtitles_job(to_convert)
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{jobs::JobSpec, testing};

    fn write_video_info(
        stash: &crate::Vault,
        repo_path_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = testing::entry_for(
            repo_path_str,
            &stash.new_file_tree(),
            &mut stash.open_catalog().expect("open_catalog"),
        )?;
        let js = crate::jobs::generate_video_info_job_spec();
        let job = js.create_job(stash, &entry)?.ok_or("job not needed")?;
        job()?;
        Ok(())
    }

    #[test]
    fn test_run_preview_job() {
        let file_root = testing::testdata_path("preview");
        let tempdir = tempfile::tempdir().expect("tempdir");
        let out_dir = tempdir.path();

        // Preview an image
        run_preview_job(&file_root.join("square.png"), &out_dir.join("square.webp")).unwrap();

        // Preview a video
        run_preview_job(
            &file_root.join("short_video.mp4"),
            &out_dir.join("short_video.webp"),
        )
        .unwrap();

        // Preview something else (should fail)
        assert!(
            run_preview_job(
                &file_root.join("make_testdata.sh"),
                &out_dir.join("make_testdata.webp")
            )
            .is_err()
        );
        assert_eq!(false, out_dir.join("make_testdata.webp").exists());

        // Make sure image preview is idempotent
        run_preview_job(&file_root.join("square.png"), &out_dir.join("square.webp")).unwrap();
    }

    #[test]
    fn test_preview_job_spec() {
        let file_root = testing::testdata_path("preview");
        let (_tempdir, vault) = testing::tempdir_vault(&file_root).expect("tempdir_stash");

        let file_tree = vault.new_file_tree();
        let mut catalog = vault.open_catalog().expect("open_catalog");
        let gen_tree = vault.new_generated_tree();
        let spec = PreviewJobSpec {};

        // Not video or image
        assert!(
            spec.create_job(&vault, &testing::fake_entry("make_testdata.sh"))
                .expect("create_job")
                .is_none()
        );

        // Regular image
        let entry = testing::entry_for("square.png", &file_tree, &mut catalog).expect("entry_for");
        spec.create_job(&vault, &entry)
            .expect("create_job")
            .expect("job needed")()
        .expect("run job");
        let preview_path = preview::get_preview(entry.db.id, &gen_tree);
        assert!(preview_path.exists());

        // Preview already exists
        assert!(
            spec.create_job(&vault, &entry)
                .expect("create_job")
                .is_none()
        );
    }

    #[test]
    #[ignore]
    fn test_transcode_job_run() -> Result<(), Box<dyn std::error::Error>> {
        let file_root = testing::testdata_path("transcode");
        let (_tempdir, stash) = testing::tempdir_vault(&file_root)?;

        let video_path = file_root.join("vidaud_h265_aac.mkv");

        let gen_tree = stash.new_generated_tree();
        run_transcode_job(&video_path, 1, &gen_tree)?;

        let files =
            gen_tree.query_generated_files(1, crate::file_tree::GeneratedFileType::AltFormat);
        assert!(files.len() > 0);

        Ok(())
    }

    #[test]
    fn test_is_transcode_needed_chrome() -> testing::TestResult {
        // Use a tempdir stash for all cases
        let file_root = testing::testdata_path("transcode");
        let (_tempdir, stash) = testing::tempdir_vault(&file_root)?;

        let file_tree = stash.new_file_tree();
        let mut catalog = stash.open_catalog().expect("open_catalog");

        // Images
        let image_entry = testing::fake_entry("an_image.jpg");
        assert_eq!(false, is_transcode_needed_chrome(&image_entry));

        // Bog standard H264-in-MP4, which Chrome can play
        write_video_info(&stash, "vidaud_h264_aac.mp4")?;
        assert_eq!(
            false,
            is_transcode_needed_chrome(&testing::entry_for(
                "vidaud_h264_aac.mp4",
                &file_tree,
                &mut catalog
            )?)
        );

        // H265-in-MKV, which Chrome can't play
        write_video_info(&stash, "vidaud_h265_aac.mkv")?;
        assert_eq!(
            true,
            is_transcode_needed_chrome(&testing::entry_for(
                "vidaud_h265_aac.mkv",
                &file_tree,
                &mut catalog
            )?)
        );

        Ok(())
    }

    #[test]
    fn test_convert_subtitle_job_spec() {
        let root = testing::testdata_path("subtitle");
        let (_tempdir, vault) = testing::tempdir_vault(&root).expect("tempdir_stash");

        let entry = testing::entry_for(
            "art001m1203451716~small_10s.mp4",
            &vault.new_file_tree(),
            &mut vault.open_catalog().expect("open_catalog"),
        )
        .expect("entry_for");

        let spec = ConvertSubtitlesJobSpec {};
        let job_maybe = spec.create_job(&vault, &entry).expect("create_job");

        assert!(job_maybe.is_some(), "Subtitle requires conversion");
        job_maybe.unwrap()().expect("run convert subtitles job");

        // Check that all subtitles converted
        let gen_tree = vault.new_generated_tree();
        let files = gen_tree
            .query_generated_files(entry.db.id, crate::file_tree::GeneratedFileType::Subtitle);
        println!("files: {:?}", files);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_convert_subtitle_job_spec_not_needed() {
        // TODO: implement
    }
}

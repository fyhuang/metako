use std::path::PathBuf;

use crate::file_tree::{GeneratedFile, GeneratedFileType, GeneratedTree};

pub const PREVIEW_IMAGE_DIMENSION: i32 = 480;
pub const PREVIEW_IMAGE_MAX_DIMENSION: i32 = 1200;

fn preview_image_gfile(entry_id: i64) -> GeneratedFile {
    GeneratedFile {
        entry_id: entry_id,
        file_type: GeneratedFileType::Preview,
        metadata: "".to_string(),
        extension: "webp".to_string(),
    }
}

/// FFmpeg arguments for scaling and saving a preview image
pub fn ffmpeg_preview_args(cmd: &mut std::process::Command) {
    // Resize the frame so that:
    // 1) The shorter dimension is ideally 300 pixels, but
    // 2) The longer dimension does not exceed 900 pixels
    let scale_arg = format!(
        "scale=w='min(iw,if(gt(iw,ih),-1,{dim}))':\
               h='min(ih,if(gt(iw,ih),{dim},-1))',\
         scale=w='min(iw,{maxd}):h=min(ih,{maxd}):\
               force_original_aspect_ratio=decrease",
        dim=crate::preview::PREVIEW_IMAGE_DIMENSION,
        maxd=crate::preview::PREVIEW_IMAGE_MAX_DIMENSION,
    );

    cmd.arg("-vf")
        .arg(scale_arg)
        // Set the jpeg quality
        /*.arg("-q:v")
        .arg("8");*/
        // Set webp settings
        .arg("-vframes")
        .arg("1")
        .arg("-compression_level")
        .arg("5")
        .arg("-quality")
        .arg("50");
}

/// Returns the path where the preview image for the given entry ID is stored.
/// The preview image does not necessarily exist.
pub fn get_preview(entry_id: i64, gen_tree: &GeneratedTree) -> PathBuf {
    let gfile = preview_image_gfile(entry_id);
    gen_tree.path_to_generated_file(&gfile)
}

#[cfg(test)]
mod tests {
    use crate::testing::testdata_path;

    use super::*;

    #[test]
    fn test_ffmpeg_resize_preview_args() {
        // Just check that the args parse successfully. We'll check the actual output
        // in the tests for image.
        let root = testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");

        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.arg("-i")
            .arg("-y")
            .arg(root.join("square.png"));
        ffmpeg_preview_args(&mut cmd);
        cmd.arg(dest_file.path())
            .output()
            .expect("failed to execute ffmpeg");
    }
}

use std::path::Path;

use crate::preview;

use super::ffmpeg;

pub fn make_preview_image(image_path: &Path, out_path: &Path) {
    let mut command = std::process::Command::new("ffmpeg");
    command.arg("-i").arg(image_path);
    preview::ffmpeg_preview_args(&mut command);
    let output = command
        .arg(out_path)
        .output()
        .expect("failed to spawn ffmpeg");
    ffmpeg::check_command_output(output, "make_preview_image");
}

#[cfg(test)]
mod tests {
    use crate::testing::testdata_path;

    use super::*;

    fn get_image_dimensions(image_path: &Path) -> (u32, u32) {
        // Use ffprobe to get the dimensions of the image
        let output = std::process::Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v:0")
            .arg("-show_entries")
            .arg("stream=width,height")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(image_path)
            .output()
            .expect("failed to execute ffprobe");

        let stdout = std::str::from_utf8(&output.stdout).expect("from_utf8");
        let mut lines = stdout.lines();
        let width = lines.next().expect("width").parse().expect("parse");
        let height = lines.next().expect("height").parse().expect("parse");
        (width, height)
    }

    #[test]
    fn test_preview_square() {
        let root = testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");

        make_preview_image(&root.join("square.png"), dest_file.path());
        assert!(dest_file.path().exists());
        assert_eq!(get_image_dimensions(dest_file.path()), (480, 480));
    }

    #[test]
    fn test_preview_wide() {
        let root = testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");

        make_preview_image(&root.join("wide.png"), dest_file.path());
        assert!(dest_file.path().exists());
        assert_eq!(get_image_dimensions(dest_file.path()), (853, 480));
    }

    #[test]
    fn test_preview_xwide() {
        let root = testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");

        make_preview_image(&root.join("xwide.png"), dest_file.path());
        assert!(dest_file.path().exists());
        assert_eq!(get_image_dimensions(dest_file.path()), (1200, 400));
    }

    #[test]
    fn test_preview_xxwide() {
        let root = testdata_path("preview");
        let dest_file = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .expect("temp file");
        std::fs::remove_file(dest_file.path()).expect("remove temp file");

        make_preview_image(&root.join("xxwide.png"), dest_file.path());
        assert!(dest_file.path().exists());
        assert_eq!(get_image_dimensions(dest_file.path()), (1200, 120));
    }
}

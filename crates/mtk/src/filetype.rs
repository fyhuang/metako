use std::path::Path;

pub const VIDEO_EXTENSIONS: [&'static str; 5] = ["mp4", "m4v", "mkv", "wmv", "webm"];
pub const IMAGE_EXTENSIONS: [&'static str; 6] = ["jpg", "jpeg", "png", "webp", "avif", "gif"];
pub const DOCUMENT_EXTENSIONS: [&'static str; 4] = ["txt", "md", "pdf", "epub"];

pub fn is_video(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        VIDEO_EXTENSIONS.iter().any(|&e| e == ext)
    } else {
        false
    }
}

pub fn is_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        IMAGE_EXTENSIONS.iter().any(|&e| e == ext)
    } else {
        false
    }
}

pub fn is_document(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        DOCUMENT_EXTENSIONS.iter().any(|&e| e == ext)
    } else {
        false
    }
}
use serde::Serialize;

use mtk::Entry;
use mtk::{catalog, file_tree};

use mtk::filetype;

fn get_display_title(entry: &Entry) -> String {
    let catalog_title = entry.db.title();
    if catalog_title.is_empty() {
        entry.fs.file_name.clone()
    } else {
        catalog_title
    }
}

#[derive(Clone, Serialize)]
pub struct EntryRenderer {
    pub repo_path: String,
    pub parent_path: String,
    pub entry_id: i64,

    pub display_title: String,
    pub file_name: String,
    // FS file type
    pub file_type: file_tree::FileType,

    // Filetype hints
    pub is_image: bool,
    pub is_video: bool,

    pub catalog: catalog::DbEntry,

    // Media type-specific fields
    pub video_stats: Option<super::VideoStatsRenderer>,
    pub video_player: Option<super::VideoPlayerRenderer>,
}

impl EntryRenderer {
    pub fn from(entry: &Entry) -> EntryRenderer {
        let repo_path = &entry.fs.repo_path.0;

        EntryRenderer {
            repo_path: repo_path.to_string(),
            parent_path: entry.fs.repo_path.parent_or_empty().0,
            entry_id: entry.db.id,
            display_title: get_display_title(entry).to_string(),
            file_name: entry.fs.repo_path.file_name().to_string(),
            file_type: entry.fs.file_type.clone(),
            is_image: filetype::is_image(&entry.fs.file_path),
            is_video: filetype::is_video(&entry.fs.file_path),
            catalog: entry.db.clone(),
            video_stats: None,
            video_player: None,
        }
    }

    pub fn render_video_stats(&mut self, entry: &Entry) {
        if self.is_video {
            self.video_stats = Some(super::VideoStatsRenderer::from(entry));
        }
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

use std::path::PathBuf;

use rocket::{http::ContentType, response::content, State};

use mtk::{file_tree::GeneratedFile, RepoPathBuf, Vault};

use crate::raw_file_responder::RawFileResponder;

fn get_raw_file_responder(path: PathBuf, stash: &Vault) -> RawFileResponder {
    let repo_path = RepoPathBuf::from(path.as_path());
    // TODO(fyhuang): tokio
    let file_tree = stash.new_file_tree();
    let fs_entry = file_tree.get_fs_entry(&repo_path).unwrap();
    let file = file_tree.open_read(&repo_path).unwrap();

    let content_type = {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "mkv" => ContentType::new("video", "webm"),
                _ => ContentType::from_extension(&ext).unwrap_or(ContentType::Plain),
            }
        } else {
            ContentType::Plain
        }
    };

    RawFileResponder {
        file: file,
        size_bytes: fs_entry.size_bytes,
        mod_time: fs_entry.mod_time,
        content_type: content_type,
        cache_control: None, // TODO: add some caching?
    }
}

#[get("/raw/<path..>")]
pub async fn raw_file_get(path: PathBuf, stash: &State<Vault>) -> RawFileResponder {
    get_raw_file_responder(path, stash)
}

#[head("/raw/<path..>")]
pub async fn raw_file_head(path: PathBuf, stash: &State<Vault>) -> RawFileResponder {
    get_raw_file_responder(path, stash)
}

use std::path::{Path, PathBuf};

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

fn local_raw_file_responder(path: &Path) -> RawFileResponder {
    let file = std::fs::File::open(path).expect("File::open");
    let metadata = std::fs::metadata(path).expect("metadata");

    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content_type = match extension.as_ref() {
        "mkv" => ContentType::new("video", "webm"),
        _ => ContentType::from_extension(&extension).unwrap_or(ContentType::Plain),
    };

    RawFileResponder {
        file: file,
        size_bytes: metadata.len(),
        mod_time: metadata.modified().expect("modified").into(),
        content_type: content_type,
        cache_control: None,
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

#[get("/generated/<entry_id>/<file_type>/<metadata>/<extension>")]
pub async fn generated_file_get(
    entry_id: i64,
    file_type: String,
    metadata: String,
    extension: String,
    stash: &State<Vault>,
) -> RawFileResponder {
    // TODO(fyhuang): tokio
    let gen_tree = stash.new_generated_tree();

    let gfile = GeneratedFile {
        entry_id: entry_id,
        file_type: mtk::file_tree::GeneratedFileType::from_two_letter_code(&file_type)
            .expect("from_two_letter_code"),
        metadata: metadata,
        extension: extension.clone(),
    };
    let fs_path = gen_tree.path_to_generated_file(&gfile);
    local_raw_file_responder(&fs_path)
}

#[get("/static/index.js")]
pub async fn static_index_js() -> content::RawJavaScript<&'static str> {
    content::RawJavaScript(include_str!("../../../frontend/dist/index.js"))
}

#[get("/static/index.js.map")]
pub async fn static_index_js_map() -> &'static str {
    include_str!("../../../frontend/dist/index.js.map")
}

#[get("/static/index.css")]
pub async fn static_index_css() -> content::RawCss<&'static str> {
    content::RawCss(include_str!("../../../frontend/dist/index.css"))
}

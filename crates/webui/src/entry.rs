use std::path::PathBuf;

use askama::Template;

use rocket::response::{content, Redirect};
use rocket::State;

use mtk::filetype;
use mtk::{Entry, RepoPathBuf, Vault};

use crate::askama_tpl;

#[get("/entry/<path..>")]
pub async fn view_entry(
    path: PathBuf,
    stash: &State<Vault>,
) -> content::RawHtml<String> {
    let mut catalog = stash.open_catalog().expect("open_catalog");
    let file_tree = stash.new_file_tree();

    let path_str = path.to_str().unwrap();
    let repo_path = RepoPathBuf::from(path_str);

    // TODO: handle 404 properly here
    let fs_entry = file_tree.get_fs_entry(&repo_path).expect("get_fs_entry");
    let db_entry = catalog.get_or_create(&fs_entry);
    let entry = Entry {
        fs: fs_entry,
        db: db_entry,
    };

    if entry.fs.file_type.is_dir {
        todo!("not implemented");
    } else {
        println!("File entry at {}", repo_path);
        let mut history_db = stash.open_history_db();
        let mut entry_renderer = askama_tpl::EntryRenderer::from(&entry);
        if filetype::is_video(&entry.fs.file_path) {
            entry_renderer.video_player = Some(askama_tpl::VideoPlayerRenderer::new(&entry));
        }
        let template = askama_tpl::ViewEntryTemplate::new(
            &stash.config,
            &entry,
            entry_renderer,
            history_db.get(entry.db.id).unwrap(),
        );
        content::RawHtml(template.render().unwrap())
    }
}

#[get("/entry_by_id/<id>")]
pub async fn view_entry_by_id(id: i64, stash: &State<Vault>) -> Redirect {
    let catalog = stash.open_catalog().expect("open_catalog");
    let repo_path = catalog.get_by_id(id).expect("get_by_id").repo_path;
    Redirect::to(uri!(view_entry(PathBuf::from(repo_path.to_string()))))
}

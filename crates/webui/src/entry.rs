use std::path::PathBuf;

use askama::Template;

use mtk::catalog::DbEntry;
use rocket::State;
use rocket::response::{Redirect, content};

use mtk::filetype;
use mtk::{Entry, RepoPathBuf, Vault};

use crate::askama_tpl;

#[get("/entry/<path..>?<layout>")]
pub async fn view_entry(
    path: PathBuf,
    layout: Option<String>,
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
        let layout =
            askama_tpl::ListingLayout::from_str(layout.as_deref().unwrap_or("compact-grid"));
        println!("Layout: {:?}", layout);
        render_dir_index(&entry, &file_tree, &mut catalog, layout)
    } else {
        println!("File entry at {}", repo_path);
        let mut history_db = stash.open_history_db();
        let mut entry_renderer = askama_tpl::EntryRenderer::from(&entry);
        if filetype::is_video(&entry.fs.file_path) {
            entry_renderer.video_player =
                Some(askama_tpl::VideoPlayerRenderer::new(&stash, &entry));
        }
        let template = askama_tpl::ViewEntryTemplate::new(
            &stash.config,
            &entry,
            entry_renderer,
            history_db.get(entry.db.id).unwrap(),
        );
        if filetype::is_image(&entry.fs.file_path) {
            // TODO(fyhuang): should we do this in JS instead?
            history_db.mark_viewed(entry.db.id, None).unwrap();
        }
        content::RawHtml(template.render().unwrap())
    }
}

fn render_dir_index(
    entry: &Entry,
    file_tree: &mtk::file_tree::FileTree,
    catalog: &mut mtk::catalog::Catalog,
    layout: askama_tpl::ListingLayout,
) -> content::RawHtml<String> {
    // TODO: use scan
    let mut dir_entries = Vec::new();
    for child_fs_entry in file_tree.listdir(&entry.fs.repo_path).expect("listdir") {
        if child_fs_entry.is_metadata_file {
            continue;
        }

        let db_entry = catalog.get_or_create(&child_fs_entry);
        if should_hide_entry(&db_entry) {
            continue;
        }

        dir_entries.push(Entry {
            fs: child_fs_entry,
            db: db_entry,
        });
    }

    let template = askama_tpl::DirIndexTemplate::new(&entry, &dir_entries, layout);
    content::RawHtml(template.render().unwrap())
}

fn should_hide_entry(entry: &DbEntry) -> bool {
    let hidden_special_entry_type = entry.special_type.as_ref().is_some_and(|t| match t {
        mtk::catalog::SpecialEntryType::SeriesDir => false,
        mtk::catalog::SpecialEntryType::GalleryDir => false,
        mtk::catalog::SpecialEntryType::MetadataFile => true,
        mtk::catalog::SpecialEntryType::PreviewFile => true,
        mtk::catalog::SpecialEntryType::AltFormatFile => true,
        mtk::catalog::SpecialEntryType::SubtitleFile => true,
    });
    entry.deleted || entry.associated_entry.is_some() || hidden_special_entry_type
}

#[get("/entry_by_id/<id>")]
pub async fn view_entry_by_id(id: i64, stash: &State<Vault>) -> Redirect {
    let catalog = stash.open_catalog().expect("open_catalog");
    let repo_path = catalog.get_by_id(id).expect("get_by_id").repo_path;
    Redirect::to(uri!(view_entry(
        PathBuf::from(repo_path.to_string()),
        Option::<String>::None
    )))
}

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to(uri!(view_entry("/", Option::<String>::None)))
}

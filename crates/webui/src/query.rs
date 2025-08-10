use std::path::PathBuf;

use askama::Template;
use rocket::response::content;
use rocket::State;

use mtk::{RepoPathBuf, Vault};
use mtk::query;

use crate::askama_tpl;

#[get("/surprise/<path..>?<mode>")]
pub async fn surprise(
    path: PathBuf,
    stash: &State<Vault>,
    mode: Option<String>,
) -> content::RawHtml<String> {
    let mut catalog = stash.open_catalog().expect("open_catalog");
    let history_db = stash.open_history_db();
    let recency_weight = if let Some(mode) = mode {
        mode == "recent"
    } else {
        false
    };

    // TODO: migrate
    let entries = query::surprise::surprise_entries(
        &RepoPathBuf::from(path.as_path()),
        &stash.new_file_tree(),
        &mut catalog,
        &history_db,
        50,
        recency_weight,
    );
    let template = askama_tpl::EntryListTemplate::new("Surprise Me", &entries, askama_tpl::ListingLayout::CompactCardGrid);
    content::RawHtml(template.render().unwrap())
}

#[get("/search?<q>")]
pub async fn search(q: String, stash: &State<Vault>) -> content::RawHtml<String> {
    let mut catalog = stash.open_catalog().expect("open_catalog");
    let entries = query::search::search(
        &stash.new_file_tree(),
        &mut catalog,
        &RepoPathBuf::from(""),
        &q,
    );
    let template = askama_tpl::EntryListTemplate::new("Search Results", &entries, askama_tpl::ListingLayout::CompactCardGrid);
    content::RawHtml(template.render().unwrap())
}

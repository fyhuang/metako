use askama::Template;

use mtk::{RepoPathBuf, Entry};

use super::filters;
use super::renderers::EntryRenderer;

#[derive(Debug, PartialEq)]
pub enum ListingLayout {
    List,
    GalleryGrid,
    CompactCardGrid,
}

impl ListingLayout {
    pub fn from_str(s: &str) -> ListingLayout {
        match s {
            "grid" => ListingLayout::GalleryGrid,
            "list" => ListingLayout::List,
            _ => ListingLayout::CompactCardGrid,
        }
    }
}

#[derive(Template)]
#[template(path = "dir_listing_partial.ask.html")]
pub struct DirListingPartial {
    pub layout: ListingLayout,
    pub entries: Vec<EntryRenderer>,
    pub entry_renderer_jsons: Vec<String>,
}

impl DirListingPartial {
    pub fn from(entries: &Vec<Entry>, layout: ListingLayout) -> DirListingPartial {
        let start_time = std::time::Instant::now();
        let renderers: Vec<_> = entries.iter().map(|entry| {
            EntryRenderer::from(&entry)
        }).collect();
        println!("Rendered {} entries in {:?}", renderers.len(), start_time.elapsed());

        DirListingPartial {
            layout: layout,
            entries: renderers.to_vec(),
            entry_renderer_jsons: renderers.iter().map(|r| {
                serde_json::to_string(&r).unwrap()
            }).collect(),
        }
    }
}

pub struct ParentCrumb {
    dir_name: String,
    repo_path: String,
}

#[derive(Template)]
#[template(path = "parent_crumbs_partial.ask.html")]
pub struct ParentCrumbsPartial {
    pub file_name: String,

    pub crumbs: Vec<ParentCrumb>,
}

impl ParentCrumbsPartial {
    pub fn from(repo_path: RepoPathBuf) -> ParentCrumbsPartial {
        // Get parents
        let mut parents = Vec::new();
        let mut curr = repo_path.clone();
        while curr.0 != "" {
            if let Some(parent) = curr.parent() {
                let dir_name = if parent.0 == "" {
                    "Top"
                } else {
                    parent.file_name()
                };

                parents.push(ParentCrumb {
                    dir_name: dir_name.to_string(),
                    repo_path: parent.0.to_string(),
                });
                curr = parent;
            }
        }

        parents.reverse();

        ParentCrumbsPartial {
            file_name: repo_path.file_name().to_string(),
            crumbs: parents,
        }
    }
}


#[derive(Template)]
#[template(path = "history_partial.ask.html")]
pub struct HistoryPartial {
    pub history: mtk::userdata::ViewHistory,
}

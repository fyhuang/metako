use askama::Template;

use mtk::FilerConfig;
use mtk::Entry;

use super::renderers;
use super::filters;
use super::partial;
use super::partial::ListingLayout;
use super::edit;

#[derive(Template)]
#[template(path = "dir_index.ask.html")]
pub struct DirIndexTemplate {
    pub entry: renderers::EntryRenderer,

    pub parent_crumbs: partial::ParentCrumbsPartial,
    pub entry_editor: edit::EntryEditorPartial,
    pub dir_listing: partial::DirListingPartial,
}

impl DirIndexTemplate {
    pub fn new(dir_entry: &Entry, contents: &Vec<Entry>, layout: ListingLayout) -> DirIndexTemplate {
        DirIndexTemplate {
            entry: renderers::EntryRenderer::from(&dir_entry),

            parent_crumbs: partial::ParentCrumbsPartial::from(dir_entry.fs.repo_path.clone()),
            entry_editor: edit::EntryEditorPartial::from(&dir_entry.db),
            dir_listing: partial::DirListingPartial::from(contents, layout),
        }
    }
}

#[derive(Template)]
#[template(path = "view_entry.ask.html")]
pub struct ViewEntryTemplate {
    pub entry: renderers::EntryRenderer,

    pub local_path_prefix: String,

    pub parent_crumbs: partial::ParentCrumbsPartial,
    pub entry_editor: edit::EntryEditorPartial,
    pub history: partial::HistoryPartial,
}

impl ViewEntryTemplate {
    pub fn new(
        config: &FilerConfig,
        entry: &Entry,
        entry_renderer: renderers::EntryRenderer,
        history: mtk::userdata::ViewHistory,
    ) -> ViewEntryTemplate {
        ViewEntryTemplate {
            entry: entry_renderer,

            // TODO(fyhuang): support more than one
            // TODO(fyhuang): strip trailing slash
            local_path_prefix: config
                .local_path_prefixes
                .iter()
                .next()
                .cloned()
                .unwrap_or("".to_string()),

            parent_crumbs: partial::ParentCrumbsPartial::from(entry.fs.repo_path.clone()),
            entry_editor: edit::EntryEditorPartial::from(&entry.db),
            history: partial::HistoryPartial { history },
        }
    }
}

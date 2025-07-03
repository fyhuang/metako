use askama::Template;

use mtk::FilerConfig;
use mtk::Entry;

use super::renderers;
use super::filters;
use super::partial;

#[derive(Template)]
#[template(path = "view_entry.ask.html")]
pub struct ViewEntryTemplate {
    pub entry: renderers::EntryRenderer,

    pub local_path_prefix: String,

    pub parent_crumbs: partial::ParentCrumbsPartial,
}

impl ViewEntryTemplate {
    pub fn new(
        config: &FilerConfig,
        entry: &Entry,
        entry_renderer: renderers::EntryRenderer,
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
        }
    }
}

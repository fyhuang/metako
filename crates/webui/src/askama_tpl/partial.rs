use askama::Template;

use mtk::RepoPathBuf;

use super::filters;

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

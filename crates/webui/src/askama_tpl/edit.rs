use askama::Template;

use mtk::catalog::DbEntry;

#[derive(Template)]
#[template(path = "entry_editor_partial.ask.html")]
pub struct EntryEditorPartial {
    pub entry_id: i64,
    pub user_json: String,
}

impl EntryEditorPartial {
    pub fn from(entry: &DbEntry) -> EntryEditorPartial {
        let user_json_str = serde_json::to_string_pretty(
            entry.all_notes_user_json()
        ).expect("serialize info_user json");

        EntryEditorPartial {
            entry_id: entry.id,
            user_json: user_json_str,
        }
    }
}

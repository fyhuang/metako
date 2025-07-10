//! API for use with HTMX

use mtk::{catalog::{edit, sqlite_catalog::WhichNotes}, Vault};
use rocket::{form::Form, response::Redirect, State};

use crate::entry;

#[derive(Debug, FromForm)]
pub struct EntryEditForm {
    // Used to detect modification conflicts. Must match the current user notes in the DB.
    original_user_notes: String,

    // What to modify

    // If present and true, replace entire user notes JSON.
    replace_entire_user: Option<bool>,
    // Otherwise, replace only this one field.
    field: Option<String>,

    // Content of the modification

    // String representation of the new content.
    contents: Option<String>,
    // For numeric fields, increment the value by this amount (negative for decrement).
    number_inc: Option<i64>,
}

#[post("/edit/<entry_id>", data = "<form>")]
pub async fn edit_entry(
    entry_id: i64,
    form: Form<EntryEditForm>,
    stash: &State<Vault>,
) -> Redirect {
    let mut catalog = stash.open_catalog().expect("open_catalog");
    let parsed_original: serde_json::Value =
        serde_json::from_str(&form.original_user_notes).unwrap();

    if form.replace_entire_user.unwrap_or(false) {
        // Replace the entire notes_user with the new JSON.
        let parsed: serde_json::Value =
            serde_json::from_str(form.contents.as_ref().unwrap()).unwrap();
        catalog.update_notes_with(entry_id, WhichNotes::User, |notes: &mut serde_json::Value| {
            if &parsed_original != notes {
                // Conflict
                // TODO: handle conflict gracefully
                panic!("Conflict");
            }

            *notes = parsed.clone()
        });
    } else {
        // Modify a specific field
        let field = form.field.as_ref().expect("field is Some").to_string();
        if let Some(contents) = &form.contents {
            // TODO: handle conflict
            catalog.set_single_note(
                entry_id,
                WhichNotes::User,
                &field,
                serde_json::from_str(&contents).unwrap(),
            );
        } else if let Some(number_inc) = form.number_inc {
            catalog.update_notes_with(entry_id, WhichNotes::User, |notes: &mut serde_json::Value| {
                if &parsed_original != notes {
                    // TODO: handle conflict gracefully
                    panic!("Conflict");
                }

                edit::increment_field_i64(notes, &field, number_inc)
                    .expect("increment_field_i64");
            });
        }
    }

    // TODO: just return a partial
    /*let template = askama_webui::EntryEditorPartial::from(&catalog.get_by_id(entry_id).expect("get_by_id"));
    content::RawHtml(template.render().unwrap())*/
    Redirect::to(uri!(entry::view_entry_by_id(entry_id)))
}

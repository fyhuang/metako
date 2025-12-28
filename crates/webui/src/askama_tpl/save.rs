use askama::Template;

#[derive(Template)]
#[template(path = "save/save_inline.frag.ask.html")]
pub struct SaveInlineFragment {
    pub current_path: String,
}

#[derive(Template)]
#[template(path = "save/save_result.frag.ask.html")]
pub struct SaveResultFragment {
    pub success: bool,
    pub message: String,
}

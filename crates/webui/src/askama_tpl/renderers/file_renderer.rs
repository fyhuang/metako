use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct RawFileRenderer {
    pub repo_path: String,
}

impl RawFileRenderer {
    pub fn url(&self) -> String {
        super::super::urlencode_parts(&format!("/raw/{}", self.repo_path))
    }
}

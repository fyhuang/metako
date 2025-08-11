use serde::Serialize;

use mtk::file_tree;

#[derive(Clone, Serialize)]
pub struct RawFileRenderer {
    pub repo_path: String,
}

impl RawFileRenderer {
    pub fn url(&self) -> String {
        super::super::urlencode_parts(&format!("/raw/{}", self.repo_path))
    }
}

#[derive(Clone, Serialize)]
pub struct GeneratedFileRenderer {
    pub entry_id: i64,
    pub file_type_2l: String,
    pub metadata: String,
    pub extension: String,
}

impl GeneratedFileRenderer {
    pub fn new(gfile: &file_tree::GeneratedFile) -> GeneratedFileRenderer {
        GeneratedFileRenderer {
            entry_id: gfile.entry_id,
            file_type_2l: gfile.file_type.to_two_letter_code().to_string(),
            metadata: gfile.metadata.clone(),
            extension: gfile.extension.clone(),
        }
    }

    pub fn url(&self) -> String {
        format!("/generated/{}/{}/{}/{}", self.entry_id, self.file_type_2l, self.metadata, self.extension)
    }
}

#[derive(Clone, Serialize)]
pub enum ServableFileRenderer {
    RawFile(RawFileRenderer),
    GeneratedFile(GeneratedFileRenderer),
}

impl ServableFileRenderer {
    pub fn url(&self) -> String {
        match self {
            ServableFileRenderer::RawFile(rf) => rf.url(),
            ServableFileRenderer::GeneratedFile(gf) => gf.url(),
        }
    }
}

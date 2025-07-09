use rocket::State;
use rocket::http::ContentType;
use rocket::response::Responder;

use mtk::Vault;
use mtk::file_tree::GeneratedFile;

use crate::raw_file_responder::RawFileResponder;

#[derive(Responder)]
pub enum RawFileOrBytesResponder {
    RawFile(RawFileResponder),
    Bytes(Vec<u8>, ContentType),
}

#[get("/preview/<entry_id>")]
pub fn preview_get<'r, 'o>(entry_id: i64, stash: &State<Vault>) -> RawFileOrBytesResponder {
    // TODO: use get_preview
    let gfile = GeneratedFile {
        entry_id: entry_id,
        file_type: mtk::file_tree::GeneratedFileType::Preview,
        metadata: "".to_string(),
        extension: "webp".to_string(),
    };

    let gen_tree = stash.new_generated_tree();
    let fs_path = gen_tree.path_to_generated_file(&gfile);
    if fs_path.exists() {
        let file = std::fs::File::open(&fs_path).expect("File::open");
        let metadata = std::fs::metadata(&fs_path).expect("metadata");

        RawFileOrBytesResponder::RawFile(RawFileResponder {
            file: file,
            size_bytes: metadata.len(),
            mod_time: metadata.modified().expect("modified").into(),
            content_type: ContentType::from_extension("webp").expect("from_extension"),
            cache_control: Some("max-age=300".to_string()),
        })
    } else {
        // TODO(fyhuang): return a transparent pixel
        // data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==
        let bytes =
            base64::decode("R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==").unwrap();
        RawFileOrBytesResponder::Bytes(bytes, ContentType::GIF)
    }
}

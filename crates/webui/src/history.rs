use rocket::serde::{json::Json, Deserialize};
use rocket::State;

use mtk::{RepoPathBuf, Vault};

#[derive(Deserialize)]
pub struct VideoHistoryRequest {
    path: String,
    current_ts: Option<i64>,
    current_ratio: Option<f32>,
}

#[post("/api/video_history", data = "<data>")]
pub async fn api_video_history(data: Json<VideoHistoryRequest>, stash: &State<Vault>) {
    let catalog = stash.open_catalog().expect("open_catalog");
    let mut history_db = stash.open_history_db();

    let video_tup = if data.current_ts.is_some() {
        Some((data.current_ts.unwrap(), data.current_ratio.unwrap()))
    } else {
        None
    };

    match catalog.path_to_id(&RepoPathBuf::from(&data.path)) {
        Some(id) => {
            history_db
                .mark_viewed(id, video_tup)
                .expect("mark_viewed");
        }
        None => {
            // TODO: return a 404
        }
    }
}

#[derive(Deserialize)]
pub struct ClearHistoryRequest {
    path: String,
}

#[post("/api/clear_history", data = "<data>")]
pub async fn api_clear_history(data: Json<ClearHistoryRequest>, stash: &State<Vault>) {
    let catalog = stash.open_catalog().expect("open_catalog");
    let mut history_db = stash.open_history_db();

    match catalog.path_to_id(&RepoPathBuf::from(&data.path)) {
        Some(id) => {
            history_db
                .clear_history(id)
                .expect("clear_history");
        }
        None => {
            // TODO: return a 404
        }
    }
}

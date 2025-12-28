//! Save URL handlers

use askama::Template;
use mtk::{save, RepoPathBuf, Vault};
use rocket::{form::Form, response::content, State};

use crate::askama_tpl::SaveResultFragment;

#[derive(Debug, FromForm)]
pub struct DownloadForm {
    url: String,
    current_path: String,
}

/// POST /xapi/save/download - Analyze and download URL
#[post("/xapi/save/download", data = "<form>")]
pub async fn download_url(
    form: Form<DownloadForm>,
    stash: &State<Vault>,
) -> content::RawHtml<String> {
    let url_str = form.url.clone();
    let current_path = form.current_path.clone();
    let file_root = stash.file_root.clone();

    // Run analyze and download in blocking thread since they're sync operations
    let result = tokio::task::spawn_blocking(move || {
        // Parse URL
        let url = match url::Url::parse(&url_str) {
            Ok(u) => u,
            Err(e) => return Err(format!("Invalid URL: {}", e)),
        };

        // Analyze
        println!("Analyzing {}", &url);
        let analysis = match save::analyze(&url) {
            Ok(a) => a,
            Err(e) => return Err(format!("Analysis failed: {:?}", e)),
        };

        if analysis.targets.is_empty() {
            return Err("No download targets found".to_string());
        }

        // Use first (recommended) target
        let target = &analysis.targets[0];

        // Determine destination directory
        let repo_path = RepoPathBuf::from(current_path.as_str());
        let dest_path = file_root.join(&repo_path.0);

        // Download
        println!("Downloading {:?} to {:?}", target, &dest_path);
        if let Err(e) = save::download(target, &dest_path) {
            return Err(format!("Download failed: {:?}", e));
        }

        Ok(())
    })
    .await;

    let (success, message) = match result {
        Ok(Ok(())) => (true, "Download complete!".to_string()),
        Ok(Err(e)) => (false, e),
        Err(e) => (false, format!("Task error: {}", e)),
    };

    let fragment = SaveResultFragment { success, message };
    content::RawHtml(fragment.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

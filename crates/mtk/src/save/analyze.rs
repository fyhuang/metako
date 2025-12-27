use std::process::Command;

use url::Url;

use crate::save::types::{AnalysisResult, SaveError, SaveTarget};

/// Analyze a URL and return possible save targets
/// Tries analyzers in order: yt-dlp → gallery-dl
pub fn analyze(url: &Url) -> Result<AnalysisResult, SaveError> {
    // Try yt-dlp first (most general video/media handler)
    if let Some(result) = analyze_ytdlp(url)? {
        return Ok(result);
    }

    // Try gallery-dl (images, galleries, social media, and direct files)
    if let Some(result) = analyze_gallerydl(url)? {
        return Ok(result);
    }

    // No downloader supports this URL
    Err(SaveError::UnsupportedUrl(format!(
        "No downloader supports this URL: {}",
        url
    )))
}

/// Try to analyze URL with yt-dlp
/// Returns None if yt-dlp can't handle the URL
pub fn analyze_ytdlp(url: &Url) -> Result<Option<AnalysisResult>, SaveError> {
    let output = Command::new("yt-dlp")
        .args(["--ignore-config", "-j", url.as_str()])
        .output()?;

    if !output.status.success() {
        // yt-dlp failed, try next analyzer
        eprintln!(
            "yt-dlp failed for {}: {}",
            url,
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let info_json: serde_json::Value = serde_json::from_str(&stdout)?;

    // Extract title if available
    let title = info_json
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Some(AnalysisResult {
        original_url: url.to_string(),
        title,
        targets: vec![SaveTarget::YtDlp {
            url: url.to_string(),
        }],
    }))
}

/// Try to analyze URL with gallery-dl
/// Returns None if gallery-dl can't handle the URL
pub fn analyze_gallerydl(url: &Url) -> Result<Option<AnalysisResult>, SaveError> {
    let output = Command::new("gallery-dl")
        .args(["--dump-json", url.as_str()])
        .output()?;

    if !output.status.success() {
        // gallery-dl failed, try next analyzer
        eprintln!(
            "gallery-dl failed for {}: {}",
            url,
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let info_json: serde_json::Value = serde_json::from_str(&stdout)?;

    // Extract title if available (may be in different fields depending on site)
    let title = info_json
        .get("title")
        .or_else(|| info_json.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Some(AnalysisResult {
        original_url: url.to_string(),
        title,
        targets: vec![SaveTarget::GalleryDl {
            url: url.to_string(),
        }],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests - require external commands
    #[test]
    #[ignore]
    fn test_analyze_ytdlp_integration() {
        let url = Url::parse("https://www.youtube.com/watch?v=jNQXAC9IVRw").unwrap();
        let result = analyze_ytdlp(&url).unwrap();

        assert!(result.is_some());
        let analysis = result.unwrap();
        assert_eq!(analysis.original_url, url.to_string());
        assert!(analysis.title.is_some());
        assert_eq!(analysis.targets.len(), 1);

        match &analysis.targets[0] {
            SaveTarget::YtDlp { url: target_url } => {
                assert_eq!(target_url, url.as_str());
            }
            _ => panic!("Expected YtDlp target"),
        }
    }

    #[test]
    #[ignore]
    fn test_analyze_gallerydl_integration() {
        let url = Url::parse("https://imgur.com/gallery/brief-history-of-imgur-EBCuk").unwrap();
        let result = analyze_gallerydl(&url).unwrap();

        assert!(result.is_some());
        let analysis = result.unwrap();
        assert_eq!(analysis.original_url, url.to_string());
        assert_eq!(analysis.targets.len(), 1);

        match &analysis.targets[0] {
            SaveTarget::GalleryDl { url: target_url } => {
                assert_eq!(target_url, url.as_str());
            }
            _ => panic!("Expected GalleryDl target"),
        }
    }

    #[test]
    #[ignore]
    fn test_analyze_integration() {
        // Test that analyze() correctly chains to yt-dlp
        let youtube_url = Url::parse("https://www.youtube.com/watch?v=jNQXAC9IVRw").unwrap();
        let result = analyze(&youtube_url).unwrap();
        assert!(matches!(result.targets[0], SaveTarget::YtDlp { .. }));

        // Test gallery-dl for direct image files
        let image_url = Url::parse("https://i.imgur.com/dEuFl5h.png").unwrap();
        let result = analyze(&image_url).unwrap();
        assert!(matches!(result.targets[0], SaveTarget::GalleryDl { .. }));
    }
}

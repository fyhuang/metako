use serde::{Deserialize, Serialize};

/// Represents what can be saved from a URL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SaveTarget {
    /// Use yt-dlp (videos, audio, some sites)
    YtDlp { url: String },
    /// Use gallery-dl (images, galleries, social media, and direct files)
    GalleryDl { url: String },
}

/// Result of analyzing a URL
/// targets[0] is always the recommended choice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub original_url: String,
    pub title: Option<String>,
    pub targets: Vec<SaveTarget>,
}

/// Errors that can occur during save operations
#[derive(Debug)]
pub enum SaveError {
    /// URL parsing failed
    InvalidUrl(String),
    /// No suitable download method found
    UnsupportedUrl(String),
    /// External command failed
    CommandFailed {
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    /// IO error
    IoError(std::io::Error),
    /// JSON parsing error
    JsonError(serde_json::Error),
    /// HTTP error
    HttpError(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            SaveError::UnsupportedUrl(msg) => write!(f, "Unsupported URL: {}", msg),
            SaveError::CommandFailed {
                command,
                exit_code,
                stderr,
            } => {
                write!(f, "Command '{}' failed", command)?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if !stderr.is_empty() {
                    write!(f, ": {}", stderr)?;
                }
                Ok(())
            }
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::JsonError(e) => write!(f, "JSON error: {}", e),
            SaveError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SaveError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::IoError(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::JsonError(err)
    }
}

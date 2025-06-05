use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum SpecialEntryType {
    // TODO: should we just use external/generated info for this?
    // Special directory types
    SeriesDir,
    GalleryDir,

    // Special file types
    MetadataFile, // Used to prefill the "external" field of associated entry
    PreviewFile, // TODO(fyhuang): is this needed? just store everything in PreviewDb
    AltFormatFile, // Alternative formats, e.g. transcoded to different codec or bitrate
    SubtitleFile, // Subtitle info (format, language defined in metadata)
}

impl std::fmt::Display for SpecialEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialEntryType::SeriesDir => write!(f, "SeriesDir"),
            SpecialEntryType::GalleryDir => write!(f, "GalleryDir"),
            SpecialEntryType::MetadataFile => write!(f, "MetadataFile"),
            SpecialEntryType::PreviewFile => write!(f, "PreviewFile"),
            SpecialEntryType::AltFormatFile => write!(f, "AltFormatFile"),
            SpecialEntryType::SubtitleFile => write!(f, "SubtitleFile"),
        }
    }
}

impl std::str::FromStr for SpecialEntryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SeriesDir" => Ok(SpecialEntryType::SeriesDir),
            "GalleryDir" => Ok(SpecialEntryType::GalleryDir),
            "MetadataFile" => Ok(SpecialEntryType::MetadataFile),
            "PreviewFile" => Ok(SpecialEntryType::PreviewFile),
            "AltFormatFile" => Ok(SpecialEntryType::AltFormatFile),
            "SubtitleFile" => Ok(SpecialEntryType::SubtitleFile),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_special_entry_type_from_str() {
        assert_eq!(SpecialEntryType::from_str("MetadataFile"), Ok(SpecialEntryType::MetadataFile));
    }
}
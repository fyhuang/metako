#[derive(Debug, PartialEq)]
pub enum GeneratedFileType {
    Preview,
    AltFormat,
    Minivid,
    Subtitle,
}

#[derive(Debug)]
pub struct GeneratedFile {
    pub entry_id: i64,
    pub file_type: GeneratedFileType,
    pub metadata: String,
    pub extension: String,
}

impl GeneratedFileType {
    pub fn to_two_letter_code(&self) -> &str {
        match self {
            GeneratedFileType::Preview => "PR",
            GeneratedFileType::AltFormat => "AF",
            GeneratedFileType::Minivid => "MV",
            GeneratedFileType::Subtitle => "ST",
        }
    }
    
    pub fn from_two_letter_code(code: &str) -> Result<GeneratedFileType, String> {
        match code {
            "PR" => Ok(GeneratedFileType::Preview),
            "AF" => Ok(GeneratedFileType::AltFormat),
            "MV" => Ok(GeneratedFileType::Minivid),
            "ST" => Ok(GeneratedFileType::Subtitle),
            _ => Err(format!("Unknown two-letter code: {}", code)),
        }
    }
}
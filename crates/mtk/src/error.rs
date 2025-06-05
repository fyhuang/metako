use std::error::Error;
use std::panic::Location;

#[derive(Debug)]
pub enum InnerError {
    RusqliteError(rusqlite::Error),
    FromSqlError(rusqlite::types::FromSqlError),
    SerdeJsonError(serde_json::Error),
    ConversionError(std::str::Utf8Error),
    DbCheckError(String),
}

#[derive(Debug)]
pub struct CatalogError {
    pub error: InnerError,
    pub location: &'static Location<'static>,
}

impl CatalogError {
    pub fn db_check_error(msg: &str) -> Self {
        Self {
            error: InnerError::DbCheckError(msg.to_string()),
            location: Location::caller(),
        }
    }
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_name = match &self.error {
            InnerError::RusqliteError(_) => "RusqliteError",
            InnerError::FromSqlError(_) => "FromSqlError",
            InnerError::SerdeJsonError(_) => "SerdeJsonError",
            InnerError::ConversionError(_) => "ConversionError",
            InnerError::DbCheckError(_) => "DbCheckError",
        };
        write!(f, "{} at {}: {:?}", error_name, self.location, self.error)
    }
}

impl Error for CatalogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.error {
            InnerError::RusqliteError(e) => Some(e),
            InnerError::FromSqlError(e) => Some(e),
            InnerError::SerdeJsonError(e) => Some(e),
            InnerError::ConversionError(e) => Some(e),
            InnerError::DbCheckError(_) => None,
        }
    }
}

impl From<rusqlite::Error> for CatalogError {
    #[track_caller]
    fn from(e: rusqlite::Error) -> Self {
        Self {
            error: InnerError::RusqliteError(e),
            location: Location::caller(),
        }
    }
}

impl From<rusqlite::types::FromSqlError> for CatalogError {
    #[track_caller]
    fn from(e: rusqlite::types::FromSqlError) -> Self {
        Self {
            error: InnerError::FromSqlError(e),
            location: Location::caller(),
        }
    }
}

impl From<serde_json::Error> for CatalogError {
    #[track_caller]
    fn from(e: serde_json::Error) -> Self {
        Self {
            error: InnerError::SerdeJsonError(e),
            location: Location::caller(),
        }
    }
}

impl From<std::str::Utf8Error> for CatalogError {
    #[track_caller]
    fn from(e: std::str::Utf8Error) -> Self {
        Self {
            error: InnerError::ConversionError(e),
            location: Location::caller(),
        }
    }
}

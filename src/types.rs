use std::ffi::OsStr;

use clap::ValueEnum;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, StackerError>;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Minified,
    Pretty,
}

#[derive(Debug, Clone, Copy, ValueEnum, Eq, PartialEq)]
pub enum StyleExtension {
    Css,
    Scss,
    Sass,
}

impl TryFrom<&OsStr> for StyleExtension {
    type Error = StackerError;

    fn try_from(value: &OsStr) -> std::result::Result<Self, Self::Error> {
        match value.to_str() {
            Some(s) => match s.to_lowercase().as_str() {
                "css" => Ok(Self::Css),
                "scss" => Ok(Self::Scss),
                "sass" => Ok(Self::Sass),
                _ => Err(StackerError::Stylesheet("Invalid extension".to_string())),
            },
            None => Err(StackerError::Stylesheet("Invalid extension".to_string())),
        }
    }
}

impl StyleExtension {
    pub fn from_os_str(os_str: Option<&OsStr>) -> Option<Self> {
        os_str.and_then(|os_str| Self::try_from(os_str).ok())
    }
}

#[derive(Debug, Error)]
pub enum StackerError {
    #[error("Failed to collect style files : {0}")]
    Collect(String),
    #[error("Failed to process SASS : {0}")]
    Sass(String),
    #[error("Failed to process CSS : {0}")]
    Stylesheet(String),
    #[error("Failed to save styles : {0}")]
    Save(String),
}

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::StripPrefixError;

#[derive(Debug)]
pub enum LoaderError {
    IoError(std::io::Error),
    StripPrefixError(StripPrefixError),
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::IoError(e) => f.write_fmt(format_args!("IO error occurred : {}", e.to_string())),
            LoaderError::StripPrefixError(e) => f.write_fmt(format_args!("Strip prefix error : {}", e.to_string())),
        }
    }
}

impl Error for LoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LoaderError::IoError(e) => Some(e),
            LoaderError::StripPrefixError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for LoaderError {
    fn from(e: std::io::Error) -> Self {
        LoaderError::IoError(e)
    }
}

impl From<StripPrefixError> for LoaderError {
    fn from(e: StripPrefixError) -> Self {
        LoaderError::StripPrefixError(e)
    }
}

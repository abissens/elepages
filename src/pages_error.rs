use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::StripPrefixError;

#[derive(Debug)]
pub enum PagesError {
    MsgError(String),
    IoError(std::io::Error),
    StripPrefixError(StripPrefixError),
    AuthorMerge,
}

impl Display for PagesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PagesError::MsgError(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::IoError(e) => f.write_fmt(format_args!("IO error occurred : {}", e.to_string())),
            PagesError::StripPrefixError(e) => f.write_fmt(format_args!("Strip prefix error : {}", e.to_string())),
            PagesError::AuthorMerge => f.write_fmt(format_args!("cannot merge authors with different names")),
        }
    }
}

impl Error for PagesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PagesError::IoError(e) => Some(e),
            PagesError::StripPrefixError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PagesError {
    fn from(e: std::io::Error) -> Self {
        PagesError::IoError(e)
    }
}

impl From<StripPrefixError> for PagesError {
    fn from(e: StripPrefixError) -> Self {
        PagesError::StripPrefixError(e)
    }
}

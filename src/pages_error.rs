use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum PagesError {
    AuthorMerge(String),
    MetadataTree(String),
    ElementNotFound(String),
    ValueParsing(String),
    Conflict(String),
    Exec(String),
}

impl Display for PagesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PagesError::AuthorMerge(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::MetadataTree(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::ElementNotFound(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::ValueParsing(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::Conflict(s) => f.write_fmt(format_args!("{}", s)),
            PagesError::Exec(s) => f.write_fmt(format_args!("{}", s)),
        }
    }
}

impl Error for PagesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct StrError(pub String);

impl StrError {
    pub fn boxed(s: &str) -> Box<dyn Error> {
        Box::new(StrError(s.to_string()))
    }
}

impl Display for StrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.0))
    }
}

impl Error for StrError {}

use std::error;
use std::fmt::Debug;
use std::io::Read;

#[derive(Clone, PartialOrd, PartialEq)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub contacts: Vec<String>,
}

#[derive(Clone, PartialOrd, PartialEq)]
pub struct Metadata {
    pub title: String,
    pub authors: Vec<Author>,
    pub tags: Vec<String>,
}

pub trait Page: Debug {
    fn path(&self) -> &[String];
    fn metadata(&self) -> Option<&Metadata>;
    fn open(&self) -> Result<Box<dyn Read>, Box<dyn error::Error>>;
}

pub trait PageBundle {
    fn pages(&self) -> &[Box<dyn Page>];
}

use std::error;
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;
use std::rc::Rc;

#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub contacts: Vec<String>,
}

#[derive(Clone, PartialOrd, PartialEq, Debug)]
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

#[derive(Debug)]
pub(crate) struct PageProxy {
    pub(crate) new_path: Option<Vec<String>>,
    pub(crate) new_metadata: Option<Metadata>,
    pub(crate) inner: Rc<dyn Page>,
}

impl Page for PageProxy {
    fn path(&self) -> &[String] {
        match &self.new_path {
            None => self.inner.path(),
            Some(p) => &p,
        }
    }

    fn metadata(&self) -> Option<&Metadata> {
        match &self.new_metadata {
            None => self.inner.metadata(),
            Some(m) => Some(&m),
        }
    }

    fn open(&self) -> Result<Box<dyn Read>, Box<dyn Error>> {
        self.inner.open()
    }
}

pub trait PageBundle {
    fn pages(&self) -> &[Rc<dyn Page>];
}

pub struct VecBundle {
    pub p: Vec<Rc<dyn Page>>,
}

impl PageBundle for VecBundle {
    fn pages(&self) -> &[Rc<dyn Page>] {
        &self.p
    }
}

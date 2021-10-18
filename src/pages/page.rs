use crate::pages_error::PagesError;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::Arc;

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct Author {
    pub name: String,
    #[serde(default = "HashSet::default")]
    pub contacts: HashSet<String>,
}

impl Eq for Author {}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Author {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Author {
    pub fn merge(&self, parent: &Self) -> Result<Self, PagesError> {
        if self.name != parent.name {
            return Err(PagesError::AuthorMerge);
        }
        let mut result = Author {
            name: self.name.clone(),
            contacts: self.contacts.clone(),
        };
        for c in &parent.contacts {
            result.contacts.insert(c.clone());
        }

        Ok(result)
    }
}

#[derive(Clone, PartialEq, Deserialize, Debug)]
pub struct Metadata {
    pub title: Option<String>,
    pub summary: Option<String>,
    #[serde(default = "HashSet::default")]
    pub authors: HashSet<Author>,
    #[serde(default = "HashSet::default")]
    pub tags: HashSet<String>,
}

impl Metadata {
    fn author_by_name(&self, name: &str) -> Option<&Author> {
        for a in &self.authors {
            if a.name == name {
                return Some(a);
            }
        }
        None
    }

    pub fn merge(&self, parent: &Self) -> Result<Self, PagesError> {
        let mut result = Metadata {
            title: self.title.clone().or_else(|| parent.title.clone()),
            summary: self.summary.clone().or_else(|| parent.summary.clone()),
            authors: self.authors.clone(),
            tags: self.tags.clone(),
        };

        for p_author in &parent.authors {
            if let Some(c) = self.author_by_name(&p_author.name) {
                result.authors.remove(c);
                result.authors.insert(c.merge(&p_author)?);
            }
        }

        for tag in &parent.tags {
            result.tags.insert(tag.clone());
        }

        Ok(result)
    }
}

pub trait Page: Debug + Send + Sync {
    fn path(&self) -> &[String];
    fn metadata(&self) -> Option<&Metadata>;
    fn open(&self) -> Result<Box<dyn Read>, PagesError>;
}

#[derive(Debug)]
pub(crate) struct PageProxy {
    pub(crate) new_path: Option<Vec<String>>,
    pub(crate) new_metadata: Option<Metadata>,
    pub(crate) inner: Arc<dyn Page>,
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

    fn open(&self) -> Result<Box<dyn Read>, PagesError> {
        self.inner.open()
    }
}

pub trait PageBundle: Send + Sync {
    fn pages(&self) -> &[Arc<dyn Page>];
}

pub struct VecBundle {
    pub p: Vec<Arc<dyn Page>>,
}

impl PageBundle for VecBundle {
    fn pages(&self) -> &[Arc<dyn Page>] {
        &self.p
    }
}

use crate::pages_error::PagesError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::Arc;

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
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
    pub fn merge(&self, parent: &Self) -> anyhow::Result<Self> {
        if self.name != parent.name {
            return Err(PagesError::AuthorMerge("cannot merge authors with different names".to_string()).into());
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

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
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

    pub fn merge(&self, parent: &Self) -> anyhow::Result<Self> {
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
    fn open(&self) -> anyhow::Result<Box<dyn Read>>;
}

pub trait ArcPage {
    fn change_path(&self, new_path: Vec<String>) -> Arc<dyn Page>;
    fn change_meta(&self, new_meta: Metadata) -> Arc<dyn Page>;
}

impl ArcPage for Arc<dyn Page> {
    fn change_path(&self, new_path: Vec<String>) -> Arc<dyn Page> {
        Arc::new(PageProxy {
            new_path: Some(new_path),
            new_metadata: None,
            inner: Arc::clone(self),
        })
    }

    fn change_meta(&self, new_meta: Metadata) -> Arc<dyn Page> {
        Arc::new(PageProxy {
            new_path: None,
            new_metadata: Some(new_meta),
            inner: Arc::clone(self),
        })
    }
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

    fn open(&self) -> anyhow::Result<Box<dyn Read>> {
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

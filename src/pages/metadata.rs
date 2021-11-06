use crate::pages_error::PagesError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

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

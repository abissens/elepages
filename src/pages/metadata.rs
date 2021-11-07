use crate::pages_error::PagesError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug)]
pub struct Author {
    pub name: String,
    #[serde(default = "HashSet::default")]
    pub contacts: HashSet<String>,
}

impl Eq for Author {}

impl PartialEq for Author {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for Author {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Author {
    pub fn merge(&self, parent: &Arc<Self>) -> anyhow::Result<Self> {
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
    pub title: Option<Arc<String>>,
    pub summary: Option<Arc<String>>,
    #[serde(default = "HashSet::default")]
    pub authors: HashSet<Arc<Author>>,
    #[serde(default = "HashSet::default")]
    pub tags: HashSet<Arc<String>>,
    pub publishing_date: Option<i64>,
    pub last_edit_date: Option<i64>,
}

impl Metadata {
    pub fn merge(&self, parent: &Self) -> anyhow::Result<Self> {
        let mut result = Metadata {
            title: self.title.clone().or_else(|| parent.title.clone()),
            summary: self.summary.clone().or_else(|| parent.summary.clone()),
            authors: self.authors.clone(),
            tags: self.tags.clone(),
            publishing_date: self.publishing_date.or(parent.publishing_date),
            last_edit_date: self.last_edit_date.or(parent.last_edit_date),
        };

        for p_author in &parent.authors {
            if let Some(c) = self.authors.get(p_author) {
                result.authors.replace(Arc::new(c.merge(&p_author)?));
            }
        }

        for tag in &parent.tags {
            result.tags.insert(tag.clone());
        }

        Ok(result)
    }
}

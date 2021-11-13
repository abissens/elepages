use crate::pages::Page;
use std::any::Any;
use std::sync::Arc;

pub trait Selector: Send + Sync {
    fn select(&self, page: &Arc<dyn Page>) -> bool;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

pub struct PathSelector {
    pub query: Vec<String>,
}

impl PathSelector {
    fn select_page(&self, path: &[String], query: &[String]) -> bool {
        if query.is_empty() {
            return path.is_empty();
        }
        if path.is_empty() {
            return false;
        }
        let item_query = query.get(0).unwrap();
        if item_query == "**" {
            let mut i = 1;
            while let Some(next_item_query) = query.get(i) {
                if next_item_query == "**" {
                    i += 1;
                    continue;
                }
                for (pos, v) in path.iter().enumerate() {
                    if self.entry_match(next_item_query, v) {
                        return self.select_page(&path[pos + 1..], &query[i + 1..]);
                    }
                }
                return false;
            }
            return true;
        }

        if self.entry_match(item_query, &path[0]) {
            return self.select_page(&path[1..], &query[1..]);
        }

        false
    }

    fn entry_match_chars(&self, ic: &[char], qc: &[char]) -> bool {
        if qc.is_empty() {
            return ic.is_empty();
        }
        if ic.is_empty() {
            return false;
        }
        let q = qc[0];
        if q == '*' {
            let mut i = 1;
            while let Some(next_q) = qc.get(i) {
                if *next_q == '*' {
                    i += 1;
                    continue;
                }
                for (pos, v) in ic.iter().enumerate() {
                    if next_q == v {
                        return self.entry_match_chars(&ic[pos + 1..], &qc[i + 1..]);
                    }
                }
                return false;
            }
            return true;
        }

        if ic[0] == qc[0] {
            return self.entry_match_chars(&ic[1..], &qc[1..]);
        }

        false
    }

    fn entry_match(&self, query: &str, path_item: &str) -> bool {
        if query == "*" || query == path_item {
            return true;
        }
        if query.contains('*') {
            let pic = path_item.chars().collect::<Vec<char>>();
            let qc = query.chars().collect::<Vec<char>>();
            return self.entry_match_chars(&pic, &qc);
        }

        false
    }
}

impl Selector for PathSelector {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        self.select_page(page.path(), &self.query)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct ExtSelector {
    pub ext: String,
}

impl Selector for ExtSelector {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        let ql = self.ext.len();
        if ql == 0 {
            return true;
        }
        let path = page.path();
        path[path.len() - 1].ends_with(&self.ext)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct TagSelector {
    pub tag: String,
}

impl Selector for TagSelector {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        if let Some(m) = page.metadata() {
            return m.tags.contains(&self.tag);
        }
        false
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct AuthorSelector {
    pub author: String,
}

impl Selector for AuthorSelector {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        if let Some(m) = page.metadata() {
            return m.authors.iter().any(|a| a.name == self.author);
        }
        false
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub enum DateQuery {
    Before(i64),
    After(i64),
    Between(i64, i64),
}

impl DateQuery {
    pub fn match_query(&self, time: &i64) -> bool {
        match self {
            DateQuery::Before(v) => time < v,
            DateQuery::After(v) => time > v,
            DateQuery::Between(v1, v2) => time > v1 && time < v2,
        }
    }
}

pub struct PublishingDateSelector {
    pub query: DateQuery,
}

impl Selector for PublishingDateSelector {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        if let Some(Some(pub_date)) = page.metadata().map(|m| &m.publishing_date) {
            return self.query.match_query(pub_date);
        }
        false
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub enum Logical {
    And(Vec<Arc<dyn Selector>>),
    Or(Vec<Arc<dyn Selector>>),
    Not(Arc<dyn Selector>),
}

impl Selector for Logical {
    fn select(&self, page: &Arc<dyn Page>) -> bool {
        match self {
            Logical::And(v) => {
                for s in v {
                    if !s.select(page) {
                        return false;
                    }
                }
                true
            }
            Logical::Or(v) => {
                for s in v {
                    if s.select(page) {
                        return true;
                    }
                }
                false
            }
            Logical::Not(s) => !s.select(page),
        }
    }
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

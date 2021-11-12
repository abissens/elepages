use crate::pages::{Page, PageBundle, VecBundle};
use std::any::Any;
use std::sync::Arc;

pub trait Selector: Send + Sync {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle>;
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
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let ql = self.query.len();
        if ql == 0 {
            return Arc::clone(bundle);
        }

        let p = bundle
            .pages()
            .iter()
            .filter_map(|p: &Arc<dyn Page>| if self.select_page(p.path(), &self.query) { Some(Arc::clone(p)) } else { None })
            .collect();
        Arc::new(VecBundle { p })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct ExtSelector {
    pub ext: String,
}

impl Selector for ExtSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let ql = self.ext.len();
        if ql == 0 {
            return Arc::clone(bundle);
        }

        let p = bundle
            .pages()
            .iter()
            .filter_map(|p: &Arc<dyn Page>| {
                let path = p.path();
                if path[path.len() - 1].ends_with(&self.ext) {
                    Some(Arc::clone(p))
                } else {
                    None
                }
            })
            .collect();

        Arc::new(VecBundle { p })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct TagSelector {
    pub tag: String,
}

impl Selector for TagSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let p = bundle
            .pages()
            .iter()
            .filter_map(|p: &Arc<dyn Page>| {
                if let Some(m) = p.metadata() {
                    if m.tags.contains(&self.tag) {
                        return Some(Arc::clone(p));
                    }
                }
                None
            })
            .collect();

        Arc::new(VecBundle { p })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

pub struct AuthorSelector {
    pub author: String,
}

impl Selector for AuthorSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let p = bundle
            .pages()
            .iter()
            .filter_map(|p: &Arc<dyn Page>| {
                if let Some(m) = p.metadata() {
                    return m.authors.iter().find_map(|a| if a.name == self.author { Some(Arc::clone(p)) } else { None });
                }
                None
            })
            .collect();

        Arc::new(VecBundle { p })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

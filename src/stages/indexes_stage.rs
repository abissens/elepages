use crate::pages::{Author, Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use serde::Serialize;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::sync::Arc;

pub struct IndexStage;

impl Stage for IndexStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut pages_by_tag: HashMap<&str, Vec<&[String]>> = HashMap::new();
        let mut pages_by_author: HashMap<&str, Vec<&[String]>> = HashMap::new();
        let mut all_pages: Vec<PageIndex> = vec![];
        let mut all_tags: HashSet<&str> = HashSet::new();
        let mut all_authors: HashSet<&Author> = HashSet::new();

        for page in bundle.pages() {
            let page_path = page.path();
            all_pages.push(PageIndex {
                path: page_path,
                metadata: page.metadata(),
            });
            if let Some(metadata) = page.metadata() {
                for tag in &metadata.tags {
                    all_tags.insert(tag);
                    pages_by_tag.entry(tag).or_insert_with(Vec::new).push(page_path);
                }

                for author in &metadata.authors {
                    all_authors.insert(author);
                    pages_by_author.entry(&author.name).or_insert_with(Vec::new).push(page_path);
                }
            }
        }

        let pages_by_tag_ser = serde_json::to_string(&pages_by_tag)?;
        let pages_by_author_ser = serde_json::to_string(&pages_by_author)?;
        let all_tags_ser = serde_json::to_string(&all_tags)?;
        let all_authors_ser = serde_json::to_string(&all_authors)?;
        let all_pages_ser = serde_json::to_string(&all_pages)?;

        Ok(Arc::new(VecBundle {
            p: vec![
                Arc::new(CursorPage {
                    value: pages_by_tag_ser,
                    path: vec!["pages_by_tag.json".to_string()],
                }),
                Arc::new(CursorPage {
                    value: pages_by_author_ser,
                    path: vec!["pages_by_author.json".to_string()],
                }),
                Arc::new(CursorPage {
                    value: all_tags_ser,
                    path: vec!["all_tags.json".to_string()],
                }),
                Arc::new(CursorPage {
                    value: all_authors_ser,
                    path: vec!["all_authors.json".to_string()],
                }),
                Arc::new(CursorPage {
                    value: all_pages_ser,
                    path: vec!["all_pages.json".to_string()],
                }),
            ],
        }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize)]
struct PageIndex<'a> {
    path: &'a [String],
    metadata: Option<&'a Metadata>,
}

#[derive(Debug)]
struct CursorPage {
    value: String,
    path: Vec<String>,
}

impl Page for CursorPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self) -> anyhow::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(self.value.clone())))
    }
}

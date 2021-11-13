use crate::pages::{Author, Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::SystemTime;

pub struct IndexStage {
    pub name: String,
}

impl Stage for IndexStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let mut pages_by_tag: HashMap<&str, Vec<&[String]>> = HashMap::new();
        let mut pages_by_author: HashMap<&str, Vec<&[String]>> = HashMap::new();
        let mut all_pages: Vec<PageIndex> = vec![];
        let mut all_tags: HashSet<&str> = HashSet::new();
        let mut all_authors: HashSet<&Author> = HashSet::new();

        for page in bundle.pages() {
            let page_path = page.path();
            all_pages.push(PageIndex {
                path: page_path,
                metadata: page.metadata().map(PageMetadata::from),
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

        let result_bundle = VecBundle {
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
        };
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        Ok((
            Arc::new(result_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

#[derive(Serialize)]
struct PageIndex<'a> {
    path: &'a [String],
    metadata: Option<PageMetadata<'a>>,
}

#[derive(Serialize)]
struct PageMetadata<'a> {
    title: Option<&'a str>,
    summary: Option<&'a str>,
    #[serde(default = "HashSet::default")]
    authors: HashSet<&'a str>,
    #[serde(default = "HashSet::default")]
    tags: HashSet<&'a str>,
    #[serde(alias = "publishingDate")]
    publishing_date: Option<&'a i64>,
    #[serde(alias = "lastEditDate")]
    last_edit_date: Option<&'a i64>,
}

impl<'a> From<&'a Metadata> for PageMetadata<'a> {
    fn from(m: &'a Metadata) -> Self {
        Self {
            title: m.title.as_ref().map(|v| v.as_str()),
            summary: m.summary.as_ref().map(|v| v.as_str()),
            authors: m.authors.iter().map(|v| v.name.as_str()).collect(),
            tags: m.tags.iter().map(|v| v.as_str()).collect(),
            publishing_date: m.publishing_date.as_ref(),
            last_edit_date: m.last_edit_date.as_ref(),
        }
    }
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

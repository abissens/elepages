use crate::config::Value;
use crate::pages::{Author, Metadata, PageBundle};
use chrono::{DateTime, Datelike, NaiveDateTime, Timelike, Utc};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct BundleIndex {
    pub all_authors: HashSet<Author>,
    pub all_tags: HashSet<String>,
    pub all_pages: Vec<PageIndex>,
    pub pages_by_author: HashMap<String, Vec<PageRef>>,
    pub pages_by_tag: HashMap<String, Vec<PageRef>>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PageRef {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PageIndex {
    pub page_ref: PageRef,
    pub metadata: Option<MetadataIndex>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DateIndex {
    pub timestamp: i64,
    pub i_year: i32,
    pub short_year: String,
    pub i_month: u32,
    pub month: String,
    pub short_month: String,
    pub long_month: String,
    pub i_day: u32,
    pub day: String,
    pub short_day: String,
    pub long_day: String,
    pub i_hour: u32,
    pub i_minute: u32,
    pub i_second: u32,
}

impl From<i64> for DateIndex {
    fn from(timestamp: i64) -> Self {
        let naive_dt = NaiveDateTime::from_timestamp(timestamp, 0);
        let utc_dt: DateTime<Utc> = DateTime::from_utc(naive_dt, Utc);

        Self {
            timestamp,
            i_year: naive_dt.year(),
            short_year: utc_dt.format("%y").to_string(),
            i_month: naive_dt.month(),
            month: utc_dt.format("%m").to_string(),
            short_month: utc_dt.format("%b").to_string(),
            long_month: utc_dt.format("%B").to_string(),
            i_day: naive_dt.day(),
            day: utc_dt.format("%d").to_string(),
            short_day: utc_dt.format("%a").to_string(),
            long_day: utc_dt.format("%A").to_string(),
            i_hour: naive_dt.hour(),
            i_minute: naive_dt.minute(),
            i_second: naive_dt.second(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetadataIndex {
    pub title: Option<String>,
    pub summary: Option<String>,
    #[serde(default = "HashSet::default")]
    pub authors: HashSet<String>,
    #[serde(default = "HashSet::default")]
    pub tags: HashSet<String>,
    #[serde(default, alias = "publishingDate")]
    pub publishing_date: Option<DateIndex>,
    #[serde(default, alias = "lastEditDate")]
    pub last_edit_date: Option<DateIndex>,
    #[serde(default = "HashMap::default")]
    pub data: HashMap<String, Value>,
}

impl From<&Metadata> for MetadataIndex {
    fn from(m: &Metadata) -> Self {
        MetadataIndex {
            title: m.title.as_ref().map(|v| v.to_string()),
            summary: m.summary.as_ref().map(|v| v.to_string()),
            authors: m.authors.iter().map(|v| v.name.to_string()).collect(),
            tags: m.tags.iter().map(|v| v.to_string()).collect(),
            publishing_date: m.publishing_date.map(DateIndex::from),
            last_edit_date: m.last_edit_date.map(DateIndex::from),
            data: m.data.clone(),
        }
    }
}
impl From<&Arc<dyn PageBundle>> for BundleIndex {
    fn from(bundle: &Arc<dyn PageBundle>) -> Self {
        let mut result = BundleIndex {
            all_authors: Default::default(),
            all_tags: Default::default(),
            all_pages: vec![],
            pages_by_author: Default::default(),
            pages_by_tag: Default::default(),
        };
        for page in bundle.pages() {
            let page_ref = PageRef { path: page.path().to_vec() };
            result.all_pages.push(PageIndex {
                page_ref: page_ref.clone(),
                metadata: page.metadata().map(MetadataIndex::from),
            });
            if let Some(metadata) = page.metadata() {
                for tag in &metadata.tags {
                    result.all_tags.insert(tag.to_string());
                    result.pages_by_tag.entry(tag.to_string()).or_insert_with(Vec::new).push(page_ref.clone());
                }

                for author in &metadata.authors {
                    result.all_authors.insert(author.as_ref().clone());
                    result.pages_by_author.entry(author.name.to_string()).or_insert_with(Vec::new).push(page_ref.clone());
                }
            }
        }
        result
    }
}

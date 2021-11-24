use crate::config::{FromValue, Value};
use crate::pages::{ArcPage, Env, MetadataIndex, PageBundle, VecBundle, DateIndex};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;
use std::collections::{HashSet, HashMap};
use serde::Serialize;

pub struct PathGenerator {
    name: String,
}

impl PathGenerator {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
impl Stage for PathGenerator {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let mut result = VecBundle { p: vec![] };
        env.print_vv(&format!("stage {}", self.name()), "path generation");

        let mut candidates = vec![];
        for page in bundle.pages() {
            if let Some(Some(path_value)) = page.metadata().map(|m| m.data.get("path")) {
                if let Ok(path) = String::from_value(path_value.clone()) {
                    candidates.push((page, path));
                    continue;
                }
            }
            result.p.push(Arc::clone(page));
        }

        if !candidates.is_empty() {
            let mut registry = Handlebars::new();
            registry.set_strict_mode(true);
            for (page, path) in candidates {
                let metadata_index = page.metadata().map(MetadataIndex::from);
                let mut rev_path = page.path().to_vec();
                rev_path.reverse();
                let path_params = match &metadata_index {
                    None => PathParams::from((page.path(), rev_path.as_slice())),
                    Some(m) => PathParams::from((page.path(), rev_path.as_slice(), m)),
                };
                let rendered_path = registry.render_template(&path, &path_params)?.split('/').map(|s| s.to_string()).collect();
                result.p.push(page.change_path(rendered_path));
            }
        }

        env.print_vv(&format!("stage {}", self.name()), "path generation ended");
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();

        Ok((
            Arc::new(result),
            ProcessingResult {
                stage_name: self.name(),
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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PathParams<'a> {
    pub title: &'a str,
    #[serde(alias = "urlTitle")]
    pub url_title: &'a str,
    pub summary: &'a str,
    #[serde(default = "HashSet::default")]
    pub authors: Option<&'a HashSet<String>>,
    #[serde(default = "HashSet::default")]
    pub tags: Option<&'a HashSet<String>>,
    pub timestamp: Option<i64>,
    pub i_year: Option<i32>,
    pub short_year: &'a str,
    pub i_month: Option<u32>,
    pub month: &'a str,
    pub short_month: &'a str,
    pub long_month: &'a str,
    pub i_day: Option<u32>,
    pub day: &'a str,
    pub short_day: &'a str,
    pub long_day: &'a str,
    pub i_hour: Option<u32>,
    pub i_minute: Option<u32>,
    pub i_second: Option<u32>,
    #[serde(default, alias = "lastEditDate")]
    pub last_edit_date: Option<&'a DateIndex>,
    #[serde(default = "HashMap::default")]
    pub data: Option<&'a HashMap<String, Value>>,
    pub path: &'a [String],
    pub rev_path: &'a [String],
}
impl <'a> From<(&'a [String], &'a [String])> for PathParams<'a> {
    fn from((path, rev_path): (&'a [String], &'a [String])) -> Self {
        Self {
            title: "",
            url_title: "",
            summary: "",
            authors: None,
            tags: None,
            timestamp: None,
            i_year: None,
            short_year: "",
            i_month: None,
            month: "",
            short_month: "",
            long_month: "",
            i_day: None,
            day: "",
            short_day: "",
            long_day: "",
            i_hour: None,
            i_minute: None,
            i_second: None,
            last_edit_date: None,
            data: None,
            path,
            rev_path
        }
    }
}
impl <'a> From<(&'a [String], &'a [String], &'a MetadataIndex)> for PathParams<'a> {
    fn from((path, rev_path, metadata_index): (&'a [String], &'a [String], &'a MetadataIndex)) -> Self {
        Self {
            title: match &metadata_index.title {
                None => "",
                Some(s) => s
            },
            url_title: match &metadata_index.url_title {
                None => "",
                Some(s) => s
            },
            summary: match &metadata_index.summary {
                None => "",
                Some(s) => s
            },
            authors: Some(&metadata_index.authors),
            tags: Some(&metadata_index.tags),
            timestamp: (&metadata_index.publishing_date).as_ref().map(|d| d.timestamp),
            i_year: (&metadata_index.publishing_date).as_ref().map(|d| d.i_year),
            short_year: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_year
            },
            i_month: (&metadata_index.publishing_date).as_ref().map(|d| d.i_month),
            month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.month
            },
            short_month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_month
            },
            long_month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.long_month
            },
            i_day: (&metadata_index.publishing_date).as_ref().map(|d| d.i_day),
            day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.day
            },
            short_day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_day
            },
            long_day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.long_day
            },
            i_hour: (&metadata_index.publishing_date).as_ref().map(|d| d.i_hour),
            i_minute: (&metadata_index.publishing_date).as_ref().map(|d| d.i_minute),
            i_second: (&metadata_index.publishing_date).as_ref().map(|d| d.i_second),
            last_edit_date: (&metadata_index).last_edit_date.as_ref(),
            data: Some(&metadata_index.data),
            path,
            rev_path
        }
    }
}
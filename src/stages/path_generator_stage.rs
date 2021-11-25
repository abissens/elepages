use crate::config::{FromValue, Value};
use crate::pages::{ArcPage, DateIndex, Env, MetadataIndex, PageBundle, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use serde::Serialize;
use std::any::Any;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

pub struct PathGenerator {
    name: String,
}

impl PathGenerator {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    fn new_path_registry() -> Handlebars<'static> {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.register_helper(
            "path_join",
            Box::new(|helper: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| {
                let source: Vec<String> = helper
                    .param(0)
                    .ok_or_else(|| RenderError::new("need at least one parameter"))?
                    .value()
                    .as_array()
                    .ok_or_else(|| RenderError::new("first param should be an array"))?
                    .iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect();

                let from = helper.param(1).map(|v| v.value().as_i64().unwrap_or(0)).unwrap_or(0);
                let to = helper
                    .param(2)
                    .map(|v| v.value().as_i64().unwrap_or_else(|| source.len() as i64))
                    .unwrap_or_else(|| source.len() as i64);

                let from = if from < 0 { cmp::max(0, (source.len() + 1) as i64 + from) as usize } else { from as usize };

                let to = if to < 0 { cmp::max(0, (source.len() + 1) as i64 + to) as usize } else { to as usize };

                let range = &source[from..to];
                out.write(&range.join("/"))?;
                Ok(())
            }),
        );
        registry
    }
}

impl Stage for PathGenerator {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
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
            let registry = PathGenerator::new_path_registry();
            for (page, path) in candidates {
                let metadata_index = page.metadata().map(MetadataIndex::from);
                let mut rev_path = page.path().to_vec();
                rev_path.reverse();
                let file_name: &str = if rev_path.is_empty() {
                    ""
                } else if let Some(i) = rev_path[0].rfind('.') {
                    &rev_path[0][0..i]
                } else {
                    &rev_path[0]
                };
                let path_params = match &metadata_index {
                    None => PathParams::from((page.path(), rev_path.as_slice(), file_name)),
                    Some(m) => PathParams::from((page.path(), rev_path.as_slice(), file_name, m)),
                };

                let rendered_path = registry.render_template(&path, &path_params)?.split('/').map(|s| s.to_string()).collect();
                result.p.push(page.change_path(rendered_path));
            }
        }

        env.print_vv(&format!("stage {}", self.name()), "path generation ended");
        let end = DateTime::<Utc>::from(SystemTime::now());

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
    #[serde(default)]
    pub last_edit_date: Option<&'a DateIndex>,
    #[serde(default = "HashMap::default")]
    pub data: Option<&'a HashMap<String, Value>>,
    pub path: &'a [String],
    pub rev_path: &'a [String],
    pub file_name: &'a str,
}
impl<'a> From<(&'a [String], &'a [String], &'a str)> for PathParams<'a> {
    fn from((path, rev_path, file_name): (&'a [String], &'a [String], &'a str)) -> Self {
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
            rev_path,
            file_name,
        }
    }
}
impl<'a> From<(&'a [String], &'a [String], &'a str, &'a MetadataIndex)> for PathParams<'a> {
    fn from((path, rev_path, file_name, metadata_index): (&'a [String], &'a [String], &'a str, &'a MetadataIndex)) -> Self {
        Self {
            title: match &metadata_index.title {
                None => "",
                Some(s) => s,
            },
            url_title: match &metadata_index.url_title {
                None => "",
                Some(s) => s,
            },
            summary: match &metadata_index.summary {
                None => "",
                Some(s) => s,
            },
            authors: Some(&metadata_index.authors),
            tags: Some(&metadata_index.tags),
            timestamp: (&metadata_index.publishing_date).as_ref().map(|d| d.timestamp),
            i_year: (&metadata_index.publishing_date).as_ref().map(|d| d.i_year),
            short_year: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_year,
            },
            i_month: (&metadata_index.publishing_date).as_ref().map(|d| d.i_month),
            month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.month,
            },
            short_month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_month,
            },
            long_month: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.long_month,
            },
            i_day: (&metadata_index.publishing_date).as_ref().map(|d| d.i_day),
            day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.day,
            },
            short_day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.short_day,
            },
            long_day: match &metadata_index.publishing_date {
                None => "",
                Some(s) => &s.long_day,
            },
            i_hour: (&metadata_index.publishing_date).as_ref().map(|d| d.i_hour),
            i_minute: (&metadata_index.publishing_date).as_ref().map(|d| d.i_minute),
            i_second: (&metadata_index.publishing_date).as_ref().map(|d| d.i_second),
            last_edit_date: metadata_index.last_edit_date.as_ref(),
            data: Some(&metadata_index.data),
            path,
            rev_path,
            file_name,
        }
    }
}

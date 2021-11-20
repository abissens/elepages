use crate::pages::{BundleIndex, Env, Metadata, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use std::any::Any;
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

    fn process(&self, _: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        env.print_vv(&format!("stage {}", self.name()), "generate index pages");
        let result_bundle = VecBundle {
            p: vec![
                Arc::new(AllPagesPage {
                    path: vec!["all_pages.json".to_string()],
                }),
                Arc::new(AllAuthorsPage {
                    path: vec!["all_authors.json".to_string()],
                }),
                Arc::new(AllTagsPage {
                    path: vec!["all_tags.json".to_string()],
                }),
                Arc::new(PagesByTagPage {
                    path: vec!["pages_by_tag.json".to_string()],
                }),
                Arc::new(PagesByAuthorPage {
                    path: vec!["pages_by_author.json".to_string()],
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

#[derive(Debug)]
struct AllPagesPage {
    path: Vec<String>,
}
#[derive(Debug)]
struct AllAuthorsPage {
    path: Vec<String>,
}
#[derive(Debug)]
struct AllTagsPage {
    path: Vec<String>,
}
#[derive(Debug)]
struct PagesByTagPage {
    path: Vec<String>,
}
#[derive(Debug)]
struct PagesByAuthorPage {
    path: Vec<String>,
}

impl Page for AllPagesPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let content = serde_json::to_string(&output_index.all_pages)?;
        Ok(Box::new(Cursor::new(content)))
    }
}

impl Page for AllAuthorsPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let content = serde_json::to_string(&output_index.all_authors)?;
        Ok(Box::new(Cursor::new(content)))
    }
}

impl Page for AllTagsPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let content = serde_json::to_string(&output_index.all_tags)?;
        Ok(Box::new(Cursor::new(content)))
    }
}

impl Page for PagesByTagPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let content = serde_json::to_string(&output_index.pages_by_tag)?;
        Ok(Box::new(Cursor::new(content)))
    }
}

impl Page for PagesByAuthorPage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let content = serde_json::to_string(&output_index.pages_by_author)?;
        Ok(Box::new(Cursor::new(content)))
    }
}

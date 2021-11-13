use crate::pages::{ArcPage, PageBundle, Selector, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug)]
pub enum CopyCut {
    Copy { name: String, selector: Arc<dyn Selector>, dest: Vec<String> },
    Move { name: String, selector: Arc<dyn Selector>, dest: Vec<String> },
    Ignore { name: String, selector: Arc<dyn Selector> },
}

impl CopyCut {
    fn select(&self, selector: &Arc<dyn Selector>, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        Arc::new(VecBundle {
            p: bundle.pages().iter().filter_map(|p| if selector.select(p) { Some(Arc::clone(p)) } else { None }).collect(),
        })
    }
}
fn join_paths(a: &[String], b: &[String]) -> Vec<String> {
    let mut result = Vec::from(a);
    result.append(&mut Vec::from(b));
    result
}

impl Stage for CopyCut {
    fn name(&self) -> String {
        match self {
            CopyCut::Copy { name, .. } => name.clone(),
            CopyCut::Move { name, .. } => name.clone(),
            CopyCut::Ignore { name, .. } => name.clone(),
        }
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let p = match self {
            CopyCut::Copy { selector, dest, .. } => {
                let mut result = bundle.pages().to_vec();
                let mut copied = self.select(selector, bundle).pages().iter().map(|p| p.change_path(join_paths(dest, p.path()))).collect();
                result.append(&mut copied);
                result
            }
            CopyCut::Move { selector, dest, .. } => {
                let selected = self.select(selector, bundle);
                let mut selected_paths: HashSet<Vec<String>> = HashSet::default();
                let mut result = vec![];
                for selected_page in selected.pages() {
                    selected_paths.insert(selected_page.path().to_vec());
                    result.push(selected_page.change_path(join_paths(dest, selected_page.path())));
                }
                let mut remaining = bundle
                    .pages()
                    .iter()
                    .filter_map(|p| if selected_paths.contains(p.path()) { None } else { Some(Arc::clone(p)) })
                    .collect();
                result.append(&mut remaining);
                result
            }
            CopyCut::Ignore { selector, .. } => {
                let selected_paths: HashSet<Vec<String>> = self.select(selector, bundle).pages().iter().map(|p| p.path().to_vec()).collect();
                bundle
                    .pages()
                    .iter()
                    .filter_map(|p| if selected_paths.contains(p.path()) { None } else { Some(Arc::clone(p)) })
                    .collect()
            }
        };
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();

        Ok((
            Arc::new(VecBundle { p }),
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

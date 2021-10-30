use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use rayon::prelude::*;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::Arc;
use regex::Regex;

pub struct ComposeStage {
    pub units: Vec<Arc<ComposeUnit>>,
    pub parallel: bool,
}

pub enum ComposeUnit {
    CreateNewSet(Arc<dyn Stage>),
    ReplaceSubSet(Box<dyn SubSetSelector>, Arc<dyn Stage>),
}

pub trait SubSetSelector: Send + Sync {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle>;
}

pub struct PrefixSelector(pub Vec<String>);

impl SubSetSelector for PrefixSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };
        for p in bundle.pages() {
            if p.path().starts_with(&self.0) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Arc::new(vec_bundle)
    }
}
pub struct RegexSelector(pub Regex);

impl SubSetSelector for RegexSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };
        for p in bundle.pages() {
            if self.0.is_match(&p.path().join("/")) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Arc::new(vec_bundle)
    }
}

pub struct ExtSelector(pub String);

impl SubSetSelector for ExtSelector {
    fn select(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };
        for p in bundle.pages() {
            let path = p.path();
            if path.len() == 0 {
                continue
            }

            if path[path.len()-1].ends_with(&self.0) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Arc::new(vec_bundle)
    }
}

struct CompositionResult {
    result: Arc<dyn PageBundle>,
    selected_set: Option<Arc<dyn PageBundle>>,
}

impl ComposeStage {
    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();

        let result: Vec<CompositionResult> = self
            .units
            .par_iter()
            .map(|unit: &Arc<ComposeUnit>| {
                let result = match unit.borrow() {
                    ComposeUnit::CreateNewSet(stage) => CompositionResult {
                        result: stage.process(&bundle)?,
                        selected_set: None,
                    },
                    ComposeUnit::ReplaceSubSet(selector, stage) => {
                        let sub_set_bundle = selector.select(&bundle);
                        CompositionResult {
                            result: stage.process(&sub_set_bundle)?,
                            selected_set: Some(sub_set_bundle),
                        }
                    }
                };
                Ok(result)
            })
            .collect::<anyhow::Result<Vec<CompositionResult>>>()?;

        for r in result {
            for page in r.result.pages() {
                vec_bundle.p.push(Arc::clone(page));
            }
            if let Some(s) = r.selected_set {
                for page in s.pages() {
                    replaced_set.insert(page.path().to_vec());
                }
            }
        }

        for p in bundle.pages() {
            if !replaced_set.contains(p.path()) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Ok(Arc::new(vec_bundle))
    }

    fn sequential_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();

        for unit in &self.units {
            match unit.borrow() {
                ComposeUnit::CreateNewSet(stage) => {
                    let mut stage_pages = stage.process(bundle)?.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
                ComposeUnit::ReplaceSubSet(selector, stage) => {
                    let sub_set_bundle = selector.select(bundle);
                    for p in sub_set_bundle.pages() {
                        replaced_set.insert(p.path().to_vec());
                    }
                    let mut stage_pages = stage.process(&sub_set_bundle)?.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
            }
        }

        for p in bundle.pages() {
            if !replaced_set.contains(p.path()) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Ok(Arc::new(vec_bundle))
    }
}

impl Stage for ComposeStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        Ok(match self.parallel {
            true => self.parallel_process(bundle)?,
            false => self.sequential_process(bundle)?,
        })
    }
}

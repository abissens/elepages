use crate::pages::{Env, Page, PageBundle, Selector, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

pub struct ComposeStage {
    pub name: String,
    pub units: Vec<Arc<ComposeUnit>>,
    pub parallel: bool,
}

pub enum ComposeUnit {
    CreateNewSet(Arc<dyn Stage>),
    ReplaceSubSet(Arc<dyn Selector>, Arc<dyn Stage>),
}

struct CompositionResult {
    result: (Arc<dyn PageBundle>, ProcessingResult),
    selected_set: Option<Arc<dyn PageBundle>>,
}

impl ComposeStage {
    fn select(&self, selector: &dyn Selector, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        Arc::new(VecBundle {
            p: bundle.pages().iter().filter_map(|p| if selector.select(p) { Some(Arc::clone(p)) } else { None }).collect(),
        })
    }

    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "start composing");
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();
        let mut sub_results = vec![];

        let result: Vec<CompositionResult> = self
            .units
            .par_iter()
            .map(|unit: &Arc<ComposeUnit>| {
                let result = match unit.borrow() {
                    ComposeUnit::CreateNewSet(stage) => CompositionResult {
                        result: stage.process(bundle, env)?,
                        selected_set: None,
                    },
                    ComposeUnit::ReplaceSubSet(selector, stage) => {
                        let sub_set_bundle = self.select(selector.as_ref(), bundle);
                        CompositionResult {
                            result: stage.process(&sub_set_bundle, env)?,
                            selected_set: Some(sub_set_bundle),
                        }
                    }
                };
                Ok(result)
            })
            .collect::<anyhow::Result<Vec<CompositionResult>>>()?;

        for r in result {
            let (bundle, p_result) = r.result;
            sub_results.push(p_result);
            for page in bundle.pages() {
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
        env.print_vv(&format!("stage {}", self.name()), "compose ended");
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            Arc::new(vec_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results,
            },
        ))
    }

    fn sequential_process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name), "start composing");
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();
        let mut sub_results = vec![];

        for unit in &self.units {
            match unit.borrow() {
                ComposeUnit::CreateNewSet(stage) => {
                    let (bundle, p_result) = stage.process(bundle, env)?;
                    sub_results.push(p_result);
                    let mut stage_pages = bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
                ComposeUnit::ReplaceSubSet(selector, stage) => {
                    let sub_set_bundle = self.select(selector.as_ref(), bundle);
                    for p in sub_set_bundle.pages() {
                        replaced_set.insert(p.path().to_vec());
                    }
                    let (bundle, p_result) = stage.process(&sub_set_bundle, env)?;
                    sub_results.push(p_result);
                    let mut stage_pages = bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
            }
        }

        for p in bundle.pages() {
            if !replaced_set.contains(p.path()) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        env.print_vv(&format!("stage {}", self.name), "compose ended");
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            Arc::new(vec_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results,
            },
        ))
    }
}

impl Stage for ComposeStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        Ok(match self.parallel {
            true => self.parallel_process(bundle, env)?,
            false => self.sequential_process(bundle, env)?,
        })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

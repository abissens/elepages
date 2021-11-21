use crate::pages::{Env, PageBundle, Selector, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

pub struct ReplaceStage {
    pub name: String,
    pub inner: Arc<dyn Stage>,
    pub selector: Arc<dyn Selector>,
}

impl Stage for ReplaceStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        env.print_vv(&format!("stage {}", self.name), "start replacing");

        let sub_set_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: bundle.pages().iter().filter_map(|p| if self.selector.select(p) { Some(Arc::clone(p)) } else { None }).collect(),
        });

        let (inner_bundle, inner_result) = self.inner.process(&sub_set_bundle, env)?;
        let mut vec_bundle = VecBundle { p: inner_bundle.pages().to_vec() };

        // Append non selected pages
        let mut replaced_set = HashSet::new();
        for page in sub_set_bundle.pages() {
            replaced_set.insert(page.path());
        }

        for p in bundle.pages() {
            if !replaced_set.contains(p.path()) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }

        env.print_vv(&format!("stage {}", self.name), "replacing ended");
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        Ok((
            Arc::new(vec_bundle),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![inner_result],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

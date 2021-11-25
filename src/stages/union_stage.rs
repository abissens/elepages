use crate::pages::{Env, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;

pub struct UnionStage {
    pub name: String,
    pub stages: Vec<Arc<dyn Stage>>,
    pub parallel: bool,
}

impl UnionStage {
    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "union stage processing started");
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut sub_results = vec![];
        let stage_pages_result: Vec<(Arc<dyn PageBundle>, ProcessingResult)> = self
            .stages
            .par_iter()
            .map(|stage: &Arc<dyn Stage>| stage.process(bundle, env))
            .collect::<anyhow::Result<Vec<(Arc<dyn PageBundle>, ProcessingResult)>>>()?;

        for (bundle, p_result) in stage_pages_result {
            sub_results.push(p_result);
            for page in bundle.pages() {
                vec_bundle.p.push(Arc::clone(page));
            }
        }
        env.print_vv(&format!("stage {}", self.name()), "union stage processing ended");
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
        env.print_vv(&format!("stage {}", self.name()), "union stage processing started");
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut sub_results = vec![];

        for stage in &self.stages {
            let (bundle, p_result) = stage.process(bundle, env)?;
            sub_results.push(p_result);

            let mut stage_pages = bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
            vec_bundle.p.append(&mut stage_pages);
        }

        let end = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "union stage processing ended");
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

impl Stage for UnionStage {
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

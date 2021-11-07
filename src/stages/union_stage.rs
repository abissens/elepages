use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use rayon::prelude::*;
use std::any::Any;
use std::sync::Arc;

pub struct UnionStage {
    pub name: String,
    pub stages: Vec<Arc<dyn Stage>>,
    pub parallel: bool,
}

impl UnionStage {
    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let stage_pages_result: Vec<Arc<dyn PageBundle>> = self
            .stages
            .par_iter()
            .map(|stage: &Arc<dyn Stage>| stage.process(&bundle))
            .collect::<anyhow::Result<Vec<Arc<dyn PageBundle>>>>()?;

        for bundle in stage_pages_result {
            for page in bundle.pages() {
                vec_bundle.p.push(Arc::clone(page));
            }
        }
        Ok(Arc::new(vec_bundle))
    }

    fn sequential_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };

        for stage in &self.stages {
            let mut stage_pages = stage.process(bundle)?.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
            vec_bundle.p.append(&mut stage_pages);
        }

        Ok(Arc::new(vec_bundle))
    }
}

impl Stage for UnionStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        Ok(match self.parallel {
            true => self.parallel_process(bundle)?,
            false => self.sequential_process(bundle)?,
        })
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

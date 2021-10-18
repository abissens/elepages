use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::sync::{mpsc, Arc};
use std::thread;

pub struct UnionStage {
    pub stages: Vec<Arc<dyn Stage>>,
    pub parallel: bool,
}

impl UnionStage {
    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };

        let (tx, rx) = mpsc::channel();
        for stage in &self.stages {
            let c_tx = tx.clone();
            let c_stage = Arc::clone(stage);
            let c_bundle = Arc::clone(bundle);
            thread::spawn(move || {
                let stage_pages_result = c_stage.process(&c_bundle).map(|bundle| bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>());
                c_tx.send(stage_pages_result).unwrap();
            });
        }
        std::mem::drop(tx);
        for stage_pages_result in rx {
            let mut r_page = stage_pages_result?;
            vec_bundle.p.append(&mut r_page);
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
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        Ok(match self.parallel {
            true => self.parallel_process(bundle)?,
            false => self.sequential_process(bundle)?,
        })
    }
}

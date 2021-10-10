use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::sync::{mpsc, Arc};
use std::thread;

pub struct UnionStage {
    pub stages: Vec<Arc<dyn Stage>>,
}

impl Stage for UnionStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };

        let (tx, rx) = mpsc::channel();
        for stage in &self.stages {
            let tx = tx.clone();
            let c_stage = Arc::clone(stage);
            let c_bundle = Arc::clone(bundle);
            thread::spawn(move || {
                let stage_pages = c_stage.process(&c_bundle).pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                tx.send(stage_pages).unwrap();
            });
        }
        std::mem::drop(tx);
        for mut r_pages in rx {
            vec_bundle.p.append(&mut r_pages);
        }
        Arc::new(vec_bundle)
    }
}

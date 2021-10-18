use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::{mpsc, Arc};
use std::thread;
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
impl ComposeStage {
    fn parallel_process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();

        let (tx_result, rx_result) = mpsc::channel();
        let (tx_replaced_set, rx_replaced_set) = mpsc::channel();

        for unit in &self.units {
            let c_tx_result = tx_result.clone();
            let c_tx_replaced_set = tx_replaced_set.clone();
            let c_unit = Arc::clone(unit);
            let c_bundle = Arc::clone(bundle);
            thread::spawn(move || match c_unit.borrow() {
                ComposeUnit::CreateNewSet(stage) => {
                    let stage_pages_result = stage.process(&c_bundle).map(|bundle| bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>());
                    c_tx_result.send(stage_pages_result).unwrap();
                }
                ComposeUnit::ReplaceSubSet(selector, stage) => {
                    let sub_set_bundle = selector.select(&c_bundle);
                    c_tx_replaced_set.send(Arc::clone(&sub_set_bundle)).unwrap();
                    let stage_pages_result = stage
                        .process(&sub_set_bundle)
                        .map(|bundle| bundle.pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>());
                    c_tx_result.send(stage_pages_result).unwrap();
                }
            });
        }

        std::mem::drop(tx_result);
        std::mem::drop(tx_replaced_set);

        for r_pages_result in rx_result {
            let mut r_pages = r_pages_result?;
            vec_bundle.p.append(&mut r_pages);
        }

        for sub_set in rx_replaced_set {
            for p in sub_set.pages() {
                replaced_set.insert(p.path().to_vec());
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

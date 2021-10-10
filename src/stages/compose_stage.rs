use crate::pages::{Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::Arc;

pub struct ComposeStage {
    pub units: Vec<ComposeUnit>,
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

impl Stage for ComposeStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle> {
        let mut vec_bundle = VecBundle { p: vec![] };
        let mut replaced_set = HashSet::new();

        for unit in &self.units {
            match unit.borrow() {
                ComposeUnit::CreateNewSet(stage) => {
                    let mut stage_pages = stage.process(bundle).pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
                ComposeUnit::ReplaceSubSet(selector, stage) => {
                    let sub_set_bundle = selector.select(bundle);
                    for p in sub_set_bundle.pages() {
                        replaced_set.insert(p.path().to_vec());
                    }
                    let mut stage_pages = stage.process(&sub_set_bundle).pages().iter().map(|p| Arc::clone(p)).collect::<Vec<Arc<dyn Page>>>();
                    vec_bundle.p.append(&mut stage_pages);
                }
            }
        }

        for p in bundle.pages() {
            if !replaced_set.contains(p.path()) {
                vec_bundle.p.push(Arc::clone(p))
            }
        }
        Arc::new(vec_bundle)
    }
}

use crate::pages::{ArcPage, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use std::any::Any;
use std::sync::Arc;

pub struct CopyStage {
    pub prefix: Vec<String>,
}

impl Stage for CopyStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let p = bundle.pages().iter().map(|p| p.change_path(join_paths(&self.prefix, p.path()))).collect::<Vec<Arc<dyn Page>>>();
        Ok(Arc::new(VecBundle { p }))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

fn join_paths(a: &[String], b: &[String]) -> Vec<String> {
    let mut result = Vec::from(a);
    result.append(&mut Vec::from(b));
    result
}

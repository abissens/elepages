use crate::pages::{ArcPage, Page, PageBundle, VecBundle};
use crate::stages::stage::Stage;
use crate::stages::ProcessingResult;
use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

pub struct CopyStage {
    pub name: String,
    pub prefix: Vec<String>,
}

impl Stage for CopyStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = Instant::now();
        let p = bundle.pages().iter().map(|p| p.change_path(join_paths(&self.prefix, p.path()))).collect::<Vec<Arc<dyn Page>>>();
        let end = Instant::now();

        Ok((
            Arc::new(VecBundle { p }),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![],
            },
        ))
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

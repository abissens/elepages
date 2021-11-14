use crate::pages::PageBundle;
use std::any::Any;
use std::sync::Arc;

pub trait Stage: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)>;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub stage_name: String,
    pub start: i64,
    pub end: i64,
    pub sub_results: Vec<ProcessingResult>,
}

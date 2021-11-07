use crate::pages::PageBundle;
use std::any::Any;
use std::sync::Arc;
use std::time::Instant;

pub trait Stage: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)>;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

pub struct ProcessingResult {
    pub stage_name: String,
    pub start: Instant,
    pub end: Instant,
    pub sub_results: Vec<ProcessingResult>,
}

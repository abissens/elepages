use crate::pages::{Env, PageBundle};
use chrono::{DateTime, Utc};
use std::any::Any;
use std::sync::Arc;

pub trait Stage: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)>;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub stage_name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub sub_results: Vec<ProcessingResult>,
}

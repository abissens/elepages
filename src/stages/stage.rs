use crate::pages::PageBundle;
use std::sync::Arc;

pub struct Pipeline {
    pub stages: Vec<Box<dyn Stage>>,
}

pub trait Stage: Send + Sync {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> Arc<dyn PageBundle>;
}

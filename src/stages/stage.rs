use crate::pages::PageBundle;
use std::any::Any;
use std::sync::Arc;

pub struct Pipeline {
    pub stages: Vec<Box<dyn Stage>>,
}

pub trait Stage: Send + Sync {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>>;
    fn as_any(&self) -> &dyn Any {
        panic!("not implemented")
    }
}

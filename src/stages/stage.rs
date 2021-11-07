use crate::pages::PageBundle;
use std::any::Any;
use std::sync::Arc;

pub trait Stage: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>>;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

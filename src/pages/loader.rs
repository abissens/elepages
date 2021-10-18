use crate::pages::page::PageBundle;
use std::sync::Arc;

pub trait Loader {
    fn load(&self) -> anyhow::Result<Arc<dyn PageBundle>>;
}

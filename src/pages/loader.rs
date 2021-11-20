use crate::pages::page::PageBundle;
use crate::pages::Env;
use std::sync::Arc;

pub trait Loader {
    fn load(&self, env: &Env) -> anyhow::Result<Arc<dyn PageBundle>>;
}

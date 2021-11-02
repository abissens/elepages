use crate::pages::PageBundle;
use std::sync::Arc;

pub trait Writer {
    fn write(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<()>;
}

use crate::pages::{Env, PageBundle};
use std::sync::Arc;

pub trait Writer {
    fn write(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<()>;
}

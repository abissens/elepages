use crate::pages::{Env, PageBundle};
use crate::stages::PageGeneratorBag;
use std::sync::Arc;

pub trait Writer {
    fn write(&self, bundle: &Arc<dyn PageBundle>, env: &Env, gen_bag: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<()>;
}

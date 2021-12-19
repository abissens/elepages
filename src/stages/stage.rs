use crate::pages::{BundleIndex, Env, Page, PageBundle};
use crate::pages_error::PagesError;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::sync::{Arc, Mutex};

pub trait Stage: Send + Sync {
    fn name(&self) -> String;
    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env, gen_bag: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)>;
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

pub trait PageGenerator: Send + Sync {
    fn yield_pages(&self, output_bundle: &BundleIndex, env: &Env) -> &[Arc<dyn Page>];
}

pub trait PageGeneratorBag: Send + Sync {
    fn push(&self, g: Arc<dyn PageGenerator>) -> anyhow::Result<()>;
    fn all(&self) -> anyhow::Result<Vec<Arc<dyn PageGenerator>>>;
}

pub struct PageGeneratorBagImpl {
    bag: Vec<Arc<dyn PageGenerator>>,
}

impl PageGeneratorBagImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Arc<dyn PageGeneratorBag> {
        Arc::new(Mutex::new(Self { bag: vec![] }))
    }
}

impl PageGeneratorBag for Mutex<PageGeneratorBagImpl> {
    fn push(&self, g: Arc<dyn PageGenerator>) -> anyhow::Result<()> {
        let mut gen = self.lock().map_err(|e| PagesError::Exec(e.to_string()))?;
        gen.bag.push(g);
        Ok(())
    }

    fn all(&self) -> anyhow::Result<Vec<Arc<dyn PageGenerator>>> {
        let gen = self.lock().map_err(|e| PagesError::Exec(e.to_string()))?;
        Ok(gen.bag.clone())
    }
}

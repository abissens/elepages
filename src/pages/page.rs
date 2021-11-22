use crate::pages::{BundleIndex, Env, Metadata, PageIndex};
use std::fmt::Debug;
use std::io::Read;
use std::sync::Arc;

pub trait Page: Debug + Send + Sync {
    fn path(&self) -> &[String];
    fn metadata(&self) -> Option<&Metadata>;
    fn open(&self, output_page: &PageIndex, output_index: &BundleIndex, env: &Env) -> anyhow::Result<Box<dyn Read>>;
}

pub trait ArcPage {
    fn change_path(&self, new_path: Vec<String>) -> Arc<dyn Page>;
    fn change_meta(&self, new_meta: Metadata) -> Arc<dyn Page>;
}

impl ArcPage for Arc<dyn Page> {
    fn change_path(&self, new_path: Vec<String>) -> Arc<dyn Page> {
        Arc::new(PageProxy {
            new_path: Some(new_path),
            new_metadata: None,
            inner: Arc::clone(self),
        })
    }

    fn change_meta(&self, new_meta: Metadata) -> Arc<dyn Page> {
        Arc::new(PageProxy {
            new_path: None,
            new_metadata: Some(new_meta),
            inner: Arc::clone(self),
        })
    }
}

#[derive(Debug)]
pub(crate) struct PageProxy {
    pub(crate) new_path: Option<Vec<String>>,
    pub(crate) new_metadata: Option<Metadata>,
    pub(crate) inner: Arc<dyn Page>,
}

impl Page for PageProxy {
    fn path(&self) -> &[String] {
        match &self.new_path {
            None => self.inner.path(),
            Some(p) => p,
        }
    }

    fn metadata(&self) -> Option<&Metadata> {
        match &self.new_metadata {
            None => self.inner.metadata(),
            Some(m) => Some(m),
        }
    }

    fn open(&self, output_page: &PageIndex, output_bundle: &BundleIndex, env: &Env) -> anyhow::Result<Box<dyn Read>> {
        self.inner.open(output_page, output_bundle, env)
    }
}

pub trait PageBundle: Send + Sync {
    fn pages(&self) -> &[Arc<dyn Page>];
}

pub struct VecBundle {
    pub p: Vec<Arc<dyn Page>>,
}

impl PageBundle for VecBundle {
    fn pages(&self) -> &[Arc<dyn Page>] {
        &self.p
    }
}

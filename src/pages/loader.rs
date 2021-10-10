use crate::pages::loader_error::LoaderError;
use crate::pages::page::PageBundle;
use std::sync::Arc;

pub trait Loader {
    fn load(&self) -> Result<Arc<dyn PageBundle>, LoaderError>;
}

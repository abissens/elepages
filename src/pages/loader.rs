use crate::pages::page::PageBundle;
use crate::pages::pages_error::PagesError;
use std::sync::Arc;

pub trait Loader {
    fn load(&self) -> Result<Arc<dyn PageBundle>, PagesError>;
}

use crate::pages::loader_error::LoaderError;
use crate::pages::page::PageBundle;

pub trait Loader {
    fn load(&self) -> Result<Box<dyn PageBundle>, LoaderError>;
}

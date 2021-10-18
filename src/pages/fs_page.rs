use crate::pages::page::{Metadata, Page};
use crate::pages_error::PagesError;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct FsPage {
    file_path: PathBuf,
    rel_path: Vec<String>,
}

impl FsPage {
    pub(super) fn new(base_path: &Path, file_path: PathBuf) -> Result<Self, PagesError> {
        let rel_path = file_path
            .strip_prefix(base_path)?
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        Ok(FsPage { file_path, rel_path })
    }
}

impl Page for FsPage {
    fn path(&self) -> &[String] {
        &self.rel_path
    }

    fn metadata(&self) -> Option<&Metadata> {
        None
    }

    fn open(&self) -> Result<Box<dyn Read>, PagesError> {
        Ok(Box::new(File::open(self.file_path.as_path())?))
    }
}

use crate::pages::{BundleIndex, Env, Metadata, Page, PageIndex};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FsPage {
    file_path: PathBuf,
    rel_path: Vec<String>,
    metadata: Option<Metadata>,
}

impl FsPage {
    pub fn new(base_path: &Path, file_path: PathBuf) -> anyhow::Result<Self> {
        let rel_path = file_path
            .strip_prefix(base_path)?
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        Ok(FsPage { file_path, rel_path, metadata: None })
    }

    pub fn new_with_metadata(base_path: &Path, file_path: PathBuf, metadata: Metadata) -> anyhow::Result<Self> {
        let rel_path = file_path
            .strip_prefix(base_path)?
            .components()
            .map(|c| c.as_os_str().to_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        Ok(FsPage {
            file_path,
            rel_path,
            metadata: Some(metadata),
        })
    }
}

impl Page for FsPage {
    fn path(&self) -> &[String] {
        &self.rel_path
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    fn open(&self, _: &PageIndex, _: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        Ok(Box::new(File::open(self.file_path.as_path())?))
    }
}

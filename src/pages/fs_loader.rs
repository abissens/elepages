use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::page::{Page, PageBundle};
use crate::pages::VecBundle;
use crate::pages_error::PagesError;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct FsLoader {
    dir_or_file: PathBuf,
}

impl FsLoader {
    pub fn new(dir: PathBuf) -> Self {
        FsLoader { dir_or_file: dir }
    }
    fn visit_dirs<T>(dir: &Path, callback: &mut T) -> Result<(), PagesError>
    where
        T: FnMut(DirEntry) -> Result<(), PagesError>,
    {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    FsLoader::visit_dirs(&path, callback)?;
                } else {
                    callback(entry)?;
                }
            }
        }
        Ok(())
    }
}

impl Loader for FsLoader {
    fn load(&self) -> Result<Arc<dyn PageBundle>, PagesError> {
        if self.dir_or_file.is_file() {
            return Ok(Arc::new(VecBundle {
                p: vec![Arc::new(FsPage::new(&self.dir_or_file, self.dir_or_file.to_path_buf())?)],
            }));
        }
        let mut pages: Vec<Arc<dyn Page>> = Vec::new();
        FsLoader::visit_dirs(&self.dir_or_file, &mut |entry| {
            pages.push(Arc::new(FsPage::new(&self.dir_or_file, entry.path())?));
            Ok(())
        })?;
        Ok(Arc::new(VecBundle { p: pages }))
    }
}

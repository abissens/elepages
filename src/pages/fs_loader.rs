use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::page::{Page, PageBundle};
use crate::pages::VecBundle;
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
    fn visit_dirs<T>(dir: &Path, callback: &mut T) -> anyhow::Result<()>
    where
        T: FnMut(DirEntry) -> anyhow::Result<()>,
    {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let is_hidden = entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false);
                if is_hidden {
                    continue;
                }
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
    fn load(&self) -> anyhow::Result<Arc<dyn PageBundle>> {
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

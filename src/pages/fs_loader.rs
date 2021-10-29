use std::path::PathBuf;
use std::sync::Arc;

use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::page::{Page, PageBundle};
use crate::pages::VecBundle;
use crate::utilities::visit_dirs;

pub struct FsLoader {
    dir_or_file: PathBuf,
}

impl FsLoader {
    pub fn new(dir: PathBuf) -> Self {
        FsLoader { dir_or_file: dir }
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
        visit_dirs(&self.dir_or_file, &mut |entry| {
            pages.push(Arc::new(FsPage::new(&self.dir_or_file, entry.path())?));
            Ok(())
        })?;
        Ok(Arc::new(VecBundle { p: pages }))
    }
}

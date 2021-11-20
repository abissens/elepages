use std::path::PathBuf;
use std::sync::Arc;

use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::page::{Page, PageBundle};
use crate::pages::{Env, VecBundle};
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
    fn load(&self, env: &Env) -> anyhow::Result<Arc<dyn PageBundle>> {
        env.print_v("FsLoader", &format!("loading {}", &self.dir_or_file.to_string_lossy()));
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
        env.print_v("FsLoader", &format!("{} loaded", &self.dir_or_file.to_string_lossy()));
        Ok(Arc::new(VecBundle { p: pages }))
    }
}

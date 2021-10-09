use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::loader_error::LoaderError;
use crate::pages::page::{Page, PageBundle};
use crate::pages::VecBundle;
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct FsLoader {
    dir_or_file: PathBuf,
}

impl FsLoader {
    pub fn new(dir: PathBuf) -> Self {
        FsLoader { dir_or_file: dir }
    }
    fn visit_dirs<T>(dir: &Path, callback: &mut T) -> Result<(), LoaderError>
    where
        T: FnMut(DirEntry) -> Result<(), LoaderError>,
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
    fn load(&self) -> Result<Box<dyn PageBundle>, LoaderError> {
        if self.dir_or_file.is_file() {
            return Ok(Box::new(VecBundle {
                p: vec![Rc::new(FsPage::new(&self.dir_or_file, self.dir_or_file.to_path_buf())?)],
            }));
        }
        let mut pages: Vec<Rc<dyn Page>> = Vec::new();
        FsLoader::visit_dirs(&self.dir_or_file, &mut |entry| {
            pages.push(Rc::new(FsPage::new(&self.dir_or_file, entry.path())?));
            Ok(())
        })?;
        Ok(Box::new(VecBundle { p: pages }))
    }
}

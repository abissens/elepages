use crate::pages::fs_page::FsPage;
use crate::pages::loader::Loader;
use crate::pages::loader_error::LoaderError;
use crate::pages::page::{Page, PageBundle};
use std::fs;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

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
                p: vec![Box::new(FsPage::new(&self.dir_or_file, self.dir_or_file.to_path_buf())?)],
            }));
        }
        let mut pages: Vec<Box<dyn Page>> = Vec::new();
        FsLoader::visit_dirs(&self.dir_or_file, &mut |entry| {
            pages.push(Box::new(FsPage::new(&self.dir_or_file, entry.path())?));
            Ok(())
        })?;
        Ok(Box::new(VecBundle { p: pages }))
    }
}

struct VecBundle {
    p: Vec<Box<dyn Page>>,
}

impl PageBundle for VecBundle {
    fn pages(&self) -> &[Box<dyn Page>] {
        &self.p
    }
}

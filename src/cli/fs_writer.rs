use crate::cli::writer::Writer;
use crate::pages::PageBundle;
use rayon::prelude::*;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

pub struct FsWriter {
    path: PathBuf,
}

impl FsWriter {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            fs::create_dir(&path)?;
        }
        Ok(Self { path })
    }
}

impl Writer for FsWriter {
    fn write(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<()> {
        let pages = bundle.pages();
        // Create directories
        for page in pages {
            let path = page.path();
            if path.len() < 2 {
                continue;
            }
            let mut file_path = self.path.clone();
            for path in &path[0..path.len() - 1] {
                file_path.push(path)
            }
            create_dir_all(&file_path)?;
        }

        // Write pages
        pages
            .par_iter()
            .map(|p| {
                let mut file_path = self.path.clone();
                let path = p.path();
                if path.is_empty() {
                    return Ok(());
                }
                for v in path {
                    file_path.push(v);
                }
                let mut file = File::create(&file_path)?;
                let mut reader = p.open()?;
                io::copy(&mut reader, &mut file)?;
                Ok(())
            })
            .collect::<anyhow::Result<Vec<()>>>()?;

        Ok(())
    }
}

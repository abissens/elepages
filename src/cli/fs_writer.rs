use crate::cli::writer::Writer;
use crate::pages::{BundleIndex, Env, PageBundle, PageIndex};
use crate::pages_error::PagesError;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs::{create_dir_all, remove_dir_all, remove_file, File};
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
    fn write(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<()> {
        env.print_v("FS Writer", "start writing pages");
        let pages = bundle.pages();
        let mut all_paths = HashSet::new();

        // Clean output directory
        if self.path.exists() {
            let existing_paths = fs::read_dir(&self.path)?;

            for path in existing_paths {
                let path = path?.path();
                env.print_vv("FS Writer", &format!("removing {}", path.to_string_lossy()));
                if path.is_dir() {
                    remove_dir_all(path)?;
                } else {
                    remove_file(path)?;
                }
            }
        }
        // Create directories
        for page in pages {
            let path = page.path();
            if all_paths.contains(path) {
                return Err(PagesError::Conflict(format!("conflicting path {}", path.join("/"))).into());
            }
            all_paths.insert(path);
            if path.len() < 2 {
                continue;
            }
            let mut file_path = self.path.clone();
            for path in &path[0..path.len() - 1] {
                file_path.push(path)
            }
            env.print_vvv("FS Writer", &format!("creating directories {}", &file_path.to_string_lossy()));
            create_dir_all(&file_path)?;
        }
        // Make output index
        let output_index = BundleIndex::from(bundle);
        // Write pages
        pages
            .par_iter()
            .map(|p| {
                let page_index = PageIndex::from(p);
                let mut file_path = self.path.clone();
                let path = p.path();
                if path.is_empty() {
                    return Ok(());
                }
                for v in path {
                    file_path.push(v);
                }
                let mut file = File::create(&file_path)?;
                let mut reader = p.open(&page_index, &output_index, env)?;
                env.print_vv("FS Writer", &format!("writing output file {}", &file_path.to_string_lossy()));
                io::copy(&mut reader, &mut file)?;
                Ok(())
            })
            .collect::<anyhow::Result<Vec<()>>>()?;

        Ok(())
    }
}

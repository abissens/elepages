use std::fs;
use std::fs::DirEntry;
use std::path::Path;

pub(crate) fn visit_dirs<T>(dir: &Path, callback: &mut T) -> anyhow::Result<()>
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
                visit_dirs(&path, callback)?;
            } else {
                callback(entry)?;
            }
        }
    }
    Ok(())
}

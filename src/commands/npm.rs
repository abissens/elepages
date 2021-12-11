use crate::pages::Env;
use crate::pages_error::PagesError;
use git2::IntoCString;
use std::env;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
pub const NPM_CMD_NAME: &str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM_CMD_NAME: &str = "npm";

pub trait NpmRunner: Sync + Send + Debug {
    fn install(&self, src: &Path, env: &Env) -> anyhow::Result<()>;
    fn run(&self, src: &Path, script: &str, env: &Env) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct DefaultNpmRunner {
    npm_path: PathBuf,
}

impl NpmRunner for DefaultNpmRunner {
    fn install(&self, src: &Path, env: &Env) -> anyhow::Result<()> {
        env.print_vv("NpmRunner", "npm i");
        let output = Command::new(self.npm_path.to_string_lossy().to_string()).arg("i").current_dir(src).output()?;
        if !output.status.success() {
            return Err(PagesError::Exec(output.stderr.into_c_string().unwrap().to_string_lossy().to_string()).into());
        }
        Ok(())
    }

    fn run(&self, src: &Path, script: &str, env: &Env) -> anyhow::Result<()> {
        env.print_vv("NpmRunner", format!("npm run {}", script).as_str());
        let output = Command::new(self.npm_path.to_string_lossy().to_string()).arg("run").arg(script).current_dir(src).output()?;
        if !output.status.success() {
            return Err(PagesError::Exec(output.stderr.into_c_string().unwrap().to_string_lossy().to_string()).into());
        }
        Ok(())
    }
}

impl DefaultNpmRunner {
    pub fn new_npm_runner() -> anyhow::Result<Box<dyn NpmRunner>> {
        if let Some(npm_path_var) = env::var_os("NPM_PATH") {
            let npm_path = PathBuf::from(npm_path_var.to_string_lossy().to_string());
            return if npm_path.is_file() {
                Ok(Box::new(DefaultNpmRunner { npm_path }))
            } else {
                Err(PagesError::ElementNotFound(format!("npm path {} not found", npm_path_var.to_string_lossy())).into())
            };
        }
        let npm_from_path_var = env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths)
                .filter_map(|dir| {
                    let full_npm_path = dir.join(&NPM_CMD_NAME);
                    if full_npm_path.is_file() {
                        Some(full_npm_path)
                    } else {
                        None
                    }
                })
                .next()
        });

        if let Some(npm_path) = npm_from_path_var {
            return Ok(Box::new(DefaultNpmRunner { npm_path }));
        }

        Err(PagesError::ElementNotFound("npm path not found".to_string()).into())
    }
}

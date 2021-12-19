use crate::commands::{DefaultNpmRunner, NpmRunner};
use crate::config::Value;
use crate::pages::{Env, FsPage, Metadata, Page};
use crate::pages_error::PagesError;
use crate::stages::{HandlebarsLookup, HandlebarsLookupResult, TemplateAsset};
use crate::utilities::visit_dirs;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;

#[derive(Debug)]
pub struct HandlebarsDir {
    pub base_path: PathBuf,
    npm_runner: Box<dyn NpmRunner>,
}

pub(crate) struct HandlebarsDirResult {
    pub(crate) registry: handlebars::Handlebars<'static>,
    pub(crate) pages: HashSet<String>,
    pub(crate) template_assets: Vec<TemplateAsset>,
    pub(crate) static_assets: Vec<Arc<dyn Page>>,
}

impl HandlebarsLookupResult for HandlebarsDirResult {
    fn clone_registry(&self) -> Handlebars<'static> {
        self.registry.clone()
    }

    fn fetch(&self, page: &Arc<dyn Page>) -> Option<String> {
        let page_path = page.path();
        let l = page_path.len();
        let mut c = path_join(page_path, l - 1);
        let mut c1 = c.clone();
        c1.push_str("page.");
        c1.push_str(&page_path[l - 1]);
        if self.pages.contains(&c1) {
            return Some(c1);
        }

        c.push_str("page");

        if self.pages.contains(&c) {
            return Some(c);
        }

        for i in (0..l - 1).rev() {
            let mut c = path_join(page_path, i);
            c.push_str("page");

            if self.pages.contains(&c) {
                return Some(c);
            }
        }

        None
    }

    fn assets(&self) -> Vec<Arc<dyn Page>> {
        self.static_assets.clone()
    }

    fn template_assets(&self) -> Vec<TemplateAsset> {
        self.template_assets.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct NodePackageJson {
    scripts: HashMap<String, String>,
    #[serde(alias = "buildOutputDir")]
    build_output_dir: Option<String>,
}

impl HandlebarsDir {
    pub fn new(base_path: PathBuf) -> anyhow::Result<HandlebarsDir> {
        return Ok(HandlebarsDir {
            base_path,
            npm_runner: DefaultNpmRunner::new_npm_runner()?,
        });
    }

    pub fn new_with_npm_runner(base_path: PathBuf, npm_runner: Box<dyn NpmRunner>) -> HandlebarsDir {
        HandlebarsDir { base_path, npm_runner }
    }

    fn get_base_path_dirs(&self) -> anyhow::Result<HashSet<String>> {
        let entries = fs::read_dir(&self.base_path)?;
        let mut result = HashSet::new();
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                if let Some(file_name) = path.file_name() {
                    result.insert(file_name.to_string_lossy().to_string());
                }
            }
        }
        Ok(result)
    }

    fn exec_node_js(&self, env: &Env) -> anyhow::Result<Option<PathBuf>> {
        let node_js_path = &self.base_path.join("package.json");
        if !node_js_path.exists() {
            return Ok(None);
        }
        let package_json: NodePackageJson = serde_json::from_reader(fs::File::open(node_js_path)?)?;
        if !package_json.scripts.contains_key("build") {
            return Ok(None);
        }

        self.npm_runner.install(&self.base_path, env)?;

        let pre_build_dirs: HashSet<String>;
        let mut build_output_dir = package_json.build_output_dir;
        if build_output_dir.is_none() {
            pre_build_dirs = self.get_base_path_dirs()?;
        } else {
            pre_build_dirs = HashSet::default();
        }

        self.npm_runner.run(&self.base_path, "build", env)?;

        if build_output_dir.is_none() {
            let post_build_dirs = self.get_base_path_dirs()?;
            let diff: HashSet<String> = &post_build_dirs - &pre_build_dirs;
            if diff.is_empty() {
                return Err(PagesError::Exec("no new folder created".to_string()).into());
            }
            if diff.len() > 1 {
                let mut diff = diff.iter().cloned().collect::<Vec<String>>();
                diff.sort();
                return Err(PagesError::Exec(format!("multiple new folders created after build : {}", diff.join(", "))).into());
            }
            build_output_dir = Some(diff.iter().next().unwrap().to_string())
        }
        let result = &self.base_path.join(build_output_dir.unwrap());
        if !result.exists() {
            return Err(PagesError::Exec(format!("build folder {} not found", result.to_string_lossy())).into());
        }
        if !result.is_dir() {
            return Err(PagesError::Exec(format!("build result {} is not a dir", result.to_string_lossy())).into());
        }

        Ok(Some(result.to_path_buf()))
    }
}

impl HandlebarsLookup for HandlebarsDir {
    fn lookup(&self, env: &Env) -> anyhow::Result<Arc<dyn HandlebarsLookupResult>> {
        let mut result = HandlebarsDirResult {
            registry: Handlebars::new(),
            pages: Default::default(),
            template_assets: Default::default(),
            static_assets: Default::default(),
        };

        let nj_build = self.exec_node_js(env)?;
        let base_path = if let Some(build_folder) = nj_build { build_folder } else { self.base_path.clone() };

        env.print_vv("HandlebarsDir", &format!("handlebars lookup from dir {}", &base_path.to_string_lossy()));
        visit_dirs(&base_path, &mut |entry| {
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(&base_path)?;
            let name = entry_path.file_name().map(|e| e.to_string_lossy()).unwrap_or_else(|| "".into());
            let ext = entry_path.extension().map(|e| e.to_string_lossy()).unwrap_or_else(|| "".into());

            if name.starts_with("page.") && ext == "hbs" {
                let template_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
                let template_name = template_name.strip_suffix(".hbs").unwrap();
                // page.hbs or page.<page name>.hbs format
                result.registry.register_template_file(template_name, entry_path)?;
                result.pages.insert(template_name.to_string());
            } else if name.starts_with("asset.") && ext == "hbs" && name.len() > 8 {
                // asset.<asset name>.hbs format
                let mut asset_path = rel_path.components().map(|c| c.as_os_str().to_str().unwrap_or_default().to_string()).collect::<Vec<_>>();
                asset_path.pop();
                asset_path.push(name[6..name.len() - 4].to_string()); // asset.<name>.hbs -> <name>
                let template_name: String = asset_path.join("/");

                result.registry.register_template_file(&template_name, entry_path)?;
                result.template_assets.push(TemplateAsset {
                    path: asset_path,
                    template_name,
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                    }),
                });
            } else if ext == "hbs" {
                let template_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
                let template_name = template_name.strip_suffix(".hbs").unwrap();
                result.registry.register_template_file(template_name, entry_path)?;
            } else {
                result.static_assets.push(Arc::new(FsPage::new_with_metadata(
                    &base_path,
                    entry_path,
                    Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                    },
                )?));
            }
            Ok(())
        })?;
        env.print_vv("HandlebarsDir", &format!("handlebars lookup from dir {} ended", &self.base_path.to_string_lossy()));
        Ok(Arc::new(result))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

fn path_join(v: &[String], i: usize) -> String {
    let mut c: String = v[0..i].join("/");
    if !c.is_empty() {
        c.push('/');
    }
    c
}

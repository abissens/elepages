use crate::commands::{DefaultNpmRunner, NpmRunner};
use crate::config::Value;
use crate::pages::{Env, Page, PageBundle, VecBundle};
use crate::pages_error::PagesError;
use crate::stages::hbs_tpl_asset::{TplAsset, TplAssetMetadata};
use crate::stages::hbs_tpl_model::TplModel;
use crate::stages::{PageGeneratorBag, ProcessingResult, Stage};
use crate::utilities::visit_dirs;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;
use std::time::SystemTime;

pub struct HbsStage {
    pub name: String,
    pub tpl_path: PathBuf,
    npm_runner: Box<dyn NpmRunner>,
}

impl Stage for HbsStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env, gen_bag: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing started");

        let tpl_model = self.make_tpl_model(env)?;

        // register generator
        gen_bag.push(Arc::new(tpl_model.clone()))?;

        // Fetch pages
        let result: Vec<Arc<dyn Page>> = bundle
            .pages()
            .iter()
            .filter_map(|p| {
                if let Some(metadata) = p.metadata() {
                    if let Some(Value::Bool(is_row)) = metadata.data.get("isRaw") {
                        if *is_row {
                            return Some(Arc::clone(p));
                        }
                    }
                }
                tpl_model.fetch(p)
            })
            .collect::<Vec<Arc<dyn Page>>>();

        env.print_vv(&format!("stage {}", self.name()), "handlebars processing ended");
        let end = DateTime::<Utc>::from(SystemTime::now());
        Ok((
            Arc::new(VecBundle { p: result }),
            ProcessingResult {
                stage_name: self.name.clone(),
                start,
                end,
                sub_results: vec![],
            },
        ))
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl HbsStage {
    pub fn new(name: String, tpl_path: PathBuf) -> anyhow::Result<HbsStage> {
        Ok(HbsStage::new_with_npm_runner(name, tpl_path, DefaultNpmRunner::new_npm_runner()?))
    }

    pub fn new_with_npm_runner(name: String, tpl_path: PathBuf, npm_runner: Box<dyn NpmRunner>) -> HbsStage {
        Self { name, tpl_path, npm_runner }
    }

    fn try_npm_build(&self, env: &Env) -> anyhow::Result<Option<PathBuf>> {
        let node_js_path = &self.tpl_path.join("package.json");
        if !node_js_path.exists() {
            return Ok(None);
        }
        let package_json: NodePackageJson = serde_json::from_reader(fs::File::open(node_js_path)?)?;
        if !package_json.scripts.contains_key("build") {
            env.print_vv(&format!("stage {}", self.name()), "build script not found in package.json");
            return Ok(None);
        }

        if package_json.build_output_dir.is_none() {
            env.print_vv(&format!("stage {}", self.name()), "buildOutputDir not found in package.json");
            return Ok(None);
        }

        self.npm_runner.install(&self.tpl_path, env)?;
        self.npm_runner.run(&self.tpl_path, "build", env)?;

        let output_dir = &self.tpl_path.join(package_json.build_output_dir.unwrap());
        if !output_dir.exists() {
            return Err(PagesError::Exec(format!("build folder {} not found", output_dir.to_string_lossy())).into());
        }
        if !output_dir.is_dir() {
            return Err(PagesError::Exec(format!("build result {} is not a dir", output_dir.to_string_lossy())).into());
        }

        Ok(Some(output_dir.to_path_buf()))
    }

    fn make_tpl_model(&self, env: &Env) -> anyhow::Result<TplModel> {
        let mut result = TplModel {
            registry: Handlebars::new(),
            pages_tpl_names: Default::default(),
            assets: Default::default(),
        };
        let mut assets_map: HashMap<String, TplAssetMetadata> = HashMap::new();
        let base_path = if let Some(npm_build_output) = self.try_npm_build(env)? {
            npm_build_output
        } else {
            self.tpl_path.clone()
        };

        env.print_vv(&format!("stage {}", self.name()), &format!("handlebars lookup from dir {}", base_path.to_string_lossy()));
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
                result.pages_tpl_names.insert(template_name.to_string());
            } else if name.starts_with("asset.") && ext == "hbs" && name.len() > 8 {
                // asset.<asset name>.hbs format
                let mut asset_path = rel_path.components().map(|c| c.as_os_str().to_str().unwrap_or_default().to_string()).collect::<Vec<_>>();
                asset_path.pop();
                asset_path.push(name[6..name.len() - 4].to_string()); // asset.<name>.hbs -> <name>
                let tpl_name: String = asset_path.join("/");

                result.registry.register_template_file(&tpl_name, entry_path)?;
                result.assets.push(TplAsset::Tpl { asset_path, tpl_name, metadata: None });
            } else if name.starts_with("asset.") && name.ends_with(".hbs.json") {
                // asset.<asset name>.hbs.json format
                let mut asset_path = rel_path.components().map(|c| c.as_os_str().to_str().unwrap_or_default().to_string()).collect::<Vec<_>>();
                asset_path.pop();
                asset_path.push(name[6..name.len() - 9].to_string()); // asset.<name>.hbs.json -> <name>
                let tpl_name: String = asset_path.join("/");
                let result = serde_json::from_reader(fs::File::open(entry_path)?)?;
                assets_map.insert(tpl_name, result);
            } else if name.starts_with("asset.") && name.ends_with(".hbs.yaml") {
                // asset.<asset name>.hbs.yaml format
                let mut asset_path = rel_path.components().map(|c| c.as_os_str().to_str().unwrap_or_default().to_string()).collect::<Vec<_>>();
                asset_path.pop();
                asset_path.push(name[6..name.len() - 9].to_string()); // asset.<name>.hbs.yaml -> <name>
                let tpl_name: String = asset_path.join("/");
                let result = serde_yaml::from_reader(fs::File::open(entry_path)?)?;
                assets_map.insert(tpl_name, result);
            } else if ext == "hbs" {
                let template_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
                let template_name = template_name.strip_suffix(".hbs").unwrap();
                result.registry.register_template_file(template_name, entry_path)?;
            } else {
                result.assets.push(TplAsset::Static {
                    base_path: base_path.to_path_buf(),
                    file_path: entry_path,
                });
            }
            Ok(())
        })?;

        for asset in &mut result.assets {
            if let TplAsset::Tpl { tpl_name, metadata, .. } = asset {
                if let Some(m) = assets_map.remove(tpl_name) {
                    *metadata = Some(m);
                }
            }
        }

        env.print_vv(&format!("stage {}", self.name()), &format!("handlebars lookup from dir {} ended", &self.tpl_path.to_string_lossy()));
        Ok(result)
    }
}

#[derive(Serialize, Deserialize)]
struct NodePackageJson {
    scripts: HashMap<String, String>,
    #[serde(alias = "buildOutputDir")]
    build_output_dir: Option<String>,
}

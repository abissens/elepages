use crate::config::Value;
use crate::pages::{BundleIndex, Env, FsPage, Metadata, Page, PageBundle, VecBundle};
use crate::stages::{HbsAsset, HbsPage, PageGenerator, PageGeneratorBag, ProcessingResult, Stage};
use crate::utilities::visit_dirs;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use std::any::Any;
use std::array::IntoIter;
use std::collections::HashSet;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;
use std::time::SystemTime;

pub struct HbsStage {
    pub name: String,
    pub tpl_path: PathBuf,
}

impl Stage for HbsStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env, gen_bag: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing started");

        let tpl_model = self.make_tpl_model(&self.tpl_path)?;

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
    fn make_tpl_model(&self, tpl_path: &Path) -> anyhow::Result<TplModel> {
        let mut result = TplModel {
            registry: Handlebars::new(),
            pages_tpl_names: Default::default(),
            assets: Default::default(),
        };

        visit_dirs(tpl_path, &mut |entry| {
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(&tpl_path)?;
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
                result.assets.insert(TplAsset::Tpl { asset_path, tpl_name });
            } else if ext == "hbs" {
                let template_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
                let template_name = template_name.strip_suffix(".hbs").unwrap();
                result.registry.register_template_file(template_name, entry_path)?;
            } else {
                result.assets.insert(TplAsset::Static {
                    base_path: tpl_path.to_path_buf(),
                    file_path: entry_path,
                });
            }
            Ok(())
        })?;

        Ok(result)
    }
}

#[derive(Clone)]
struct TplModel {
    registry: handlebars::Handlebars<'static>,
    pages_tpl_names: HashSet<String>,
    assets: HashSet<TplAsset>,
}

#[derive(Eq, PartialEq, Hash, Clone)]
enum TplAsset {
    Tpl { asset_path: Vec<String>, tpl_name: String },
    Static { base_path: PathBuf, file_path: PathBuf },
}

impl PageGenerator for TplModel {
    fn yield_pages(&self, _: &BundleIndex, _: &Env) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let mut result = vec![];
        for asset in &self.assets {
            match asset {
                TplAsset::Tpl { tpl_name, asset_path } => {
                    result.push(Arc::new(HbsAsset {
                        registry: self.registry.clone(),
                        tpl_name: tpl_name.clone(),
                        path: asset_path.clone(),
                        metadata: Some(Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                        }),
                    }) as Arc<dyn Page>);
                }
                TplAsset::Static { base_path, file_path } => {
                    result.push(Arc::new(FsPage::new_with_metadata(
                        base_path,
                        file_path.to_path_buf(),
                        Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                        },
                    )?) as Arc<dyn Page>);
                }
            }
        }
        Ok(result)
    }
}

impl TplModel {
    fn fetch(&self, page: &Arc<dyn Page>) -> Option<Arc<dyn Page>> {
        self.fetch_page_tpl_name(page).map(|tpl_name| {
            Arc::new(HbsPage {
                registry: self.registry.clone(),
                source: Arc::clone(page),
                tpl_name,
            }) as Arc<dyn Page>
        })
    }

    fn fetch_page_tpl_name(&self, page: &Arc<dyn Page>) -> Option<String> {
        let page_path = page.path();
        let l = page_path.len();
        let mut c = path_join(page_path, l - 1);
        let mut c1 = c.clone();
        c1.push_str("page.");
        c1.push_str(&page_path[l - 1]);
        if self.pages_tpl_names.contains(&c1) {
            return Some(c1);
        }

        c.push_str("page");

        if self.pages_tpl_names.contains(&c) {
            return Some(c);
        }

        for i in (0..l - 1).rev() {
            let mut c = path_join(page_path, i);
            c.push_str("page");

            if self.pages_tpl_names.contains(&c) {
                return Some(c);
            }
        }

        None
    }
}

fn path_join(v: &[String], i: usize) -> String {
    let mut c: String = v[0..i].join("/");
    if !c.is_empty() {
        c.push('/');
    }
    c
}

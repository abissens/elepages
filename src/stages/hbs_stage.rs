use crate::commands::{DefaultNpmRunner, NpmRunner};
use crate::config::Value;
use crate::pages::{BundleIndex, BundlePagination, BundleQuery, Env, FsPage, Metadata, Page, PageBundle, PageIndex, VecBundle};
use crate::pages_error::PagesError;
use crate::stages::hbs_asset::{HbsAsset, HbsAssetSelection};
use crate::stages::{HbsPage, PageGenerator, PageGeneratorBag, ProcessingResult, Stage};
use crate::utilities::visit_dirs;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
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

#[derive(Clone)]
struct TplModel {
    registry: handlebars::Handlebars<'static>,
    pages_tpl_names: HashSet<String>,
    assets: Vec<TplAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum TplAssetGroupBy {
    #[serde(alias = "tag")]
    Tag,
    #[serde(alias = "author")]
    Author,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TplAssetMetadata {
    #[serde(alias = "query")]
    base_query: Option<BundleQuery>,
    #[serde(alias = "groupBy")]
    group_by: Option<TplAssetGroupBy>,
    #[serde(alias = "limit")]
    limit: Option<usize>,
    #[serde(alias = "path")]
    path_pattern: Option<String>,
    #[serde(alias = "firstPagePath")]
    first_page_path_pattern: Option<String>,
}

impl TplAssetMetadata {
    fn make_path(&self, asset_path: &[String], selection: &HbsAssetSelection) -> anyhow::Result<Vec<String>> {
        if selection.index == 0 {
            if let Some(pattern) = &self.first_page_path_pattern {
                return Ok(Handlebars::new().render_template(pattern, selection)?.split('/').map(|s| s.to_string()).collect());
            }
        }
        if let Some(pattern) = &self.path_pattern {
            return Ok(Handlebars::new().render_template(pattern, selection)?.split('/').map(|s| s.to_string()).collect());
        }

        Ok(asset_path.to_vec())
    }

    fn yield_pages(&self, registry: &handlebars::Handlebars<'static>, asset_path: &[String], tpl_name: &str, output_bundle: &BundleIndex) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let base_query = self.base_query.clone().unwrap_or(BundleQuery::Always);
        let mut queries = vec![];

        if let Some(group_by) = &self.group_by {
            match group_by {
                TplAssetGroupBy::Tag => queries.append(
                    &mut output_bundle
                        .all_tags
                        .iter()
                        .map(|t| {
                            (
                                BundleQuery::And {
                                    and: vec![base_query.clone(), BundleQuery::Tag { tag: t.to_string() }],
                                },
                                Some(t.to_string()),
                                None,
                            )
                        })
                        .collect(),
                ),
                TplAssetGroupBy::Author => queries.append(
                    &mut output_bundle
                        .all_authors
                        .iter()
                        .map(|a| {
                            (
                                BundleQuery::And {
                                    and: vec![base_query.clone(), BundleQuery::Author { author: a.name.to_string() }],
                                },
                                None,
                                Some(a.name.to_string()),
                            )
                        })
                        .collect(),
                ),
            }
        }

        if queries.is_empty() {
            queries.push((base_query, None, None));
        }
        let no_paginate = BundlePagination { skip: None, limit: None };
        if let Some(limit) = self.limit {
            let mut result = vec![];
            for (q, selection_tag, selection_author) in &queries {
                let pages_size = output_bundle.count(q, &no_paginate);
                let nb_pages: usize = (pages_size as f32 / limit as f32).ceil() as usize;
                for p in 0..nb_pages {
                    let pages = output_bundle.query(
                        q,
                        &BundlePagination {
                            skip: Some(p * limit),
                            limit: Some(limit),
                        },
                    );
                    let selection = HbsAssetSelection {
                        pages: pages
                            .iter()
                            .map(|e| PageIndex {
                                page_ref: e.page_ref.clone(),
                                page_uri: e.page_uri.clone(),
                                metadata: e.metadata.clone(),
                            })
                            .collect::<Vec<PageIndex>>(),
                        index: p,
                        limit,
                        last: nb_pages - 1,
                        size: Some(pages_size),
                        tag: selection_tag.clone(),
                        author: selection_author.clone(),
                    };

                    result.push(Arc::new(HbsAsset {
                        registry: registry.clone(),
                        tpl_name: tpl_name.to_string(),
                        path: self.make_path(asset_path, &selection)?,
                        metadata: Some(Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                        }),
                        selection: Some(selection),
                    }) as Arc<dyn Page>);
                }
            }

            return Ok(result);
        }

        let result = queries
            .iter()
            .map(|(q, selection_tag, selection_author)| {
                let pages: Vec<PageIndex> = output_bundle
                    .query(q, &no_paginate)
                    .iter()
                    .map(|e| PageIndex {
                        page_ref: e.page_ref.clone(),
                        page_uri: e.page_uri.clone(),
                        metadata: e.metadata.clone(),
                    })
                    .collect();
                let limit = pages.len();
                let selection = HbsAssetSelection {
                    pages,
                    index: 0,
                    limit,
                    last: 0,
                    size: None,
                    tag: selection_tag.clone(),
                    author: selection_author.clone(),
                };

                Ok(Arc::new(HbsAsset {
                    registry: registry.clone(),
                    tpl_name: tpl_name.to_string(),
                    path: self.make_path(asset_path, &selection)?,
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))]).collect(),
                    }),
                    selection: Some(selection),
                }) as Arc<dyn Page>)
            })
            .collect::<anyhow::Result<Vec<Arc<dyn Page>>>>()?;

        Ok(result)
    }
}
#[derive(Debug, Clone, PartialEq)]
enum TplAsset {
    Tpl {
        asset_path: Vec<String>,
        tpl_name: String,
        metadata: Option<TplAssetMetadata>,
    },
    Static {
        base_path: PathBuf,
        file_path: PathBuf,
    },
}

impl PageGenerator for TplModel {
    fn yield_pages(&self, output_bundle: &BundleIndex, _: &Env) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let mut result = vec![];
        for asset in &self.assets {
            match asset {
                TplAsset::Tpl { tpl_name, asset_path, metadata } => {
                    if let Some(tpl_meta) = metadata {
                        let mut pages = tpl_meta.yield_pages(&self.registry, asset_path, tpl_name, output_bundle)?;
                        result.append(&mut pages);
                    } else {
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
                            selection: None,
                        }) as Arc<dyn Page>);
                    }
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

#[derive(Serialize, Deserialize)]
struct NodePackageJson {
    scripts: HashMap<String, String>,
    #[serde(alias = "buildOutputDir")]
    build_output_dir: Option<String>,
}

fn path_join(v: &[String], i: usize) -> String {
    let mut c: String = v[0..i].join("/");
    if !c.is_empty() {
        c.push('/');
    }
    c
}

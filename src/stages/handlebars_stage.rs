use crate::pages::{BundleIndex, Env, FsPage, Metadata, Page, PageBundle, PageIndex, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use crate::utilities::visit_dirs;
use chrono::{DateTime, Utc};
use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use rayon::prelude::*;
use serde::Serialize;
use std::any::Any;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;
use std::time::SystemTime;

pub trait HandlebarsLookup: Sync + Send + Debug {
    fn lookup(&self, env: &Env) -> anyhow::Result<Arc<dyn HandlebarsLookupResult>>;
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
}

pub trait HandlebarsLookupResult: Send + Sync {
    fn clone_registry(&self) -> handlebars::Handlebars<'static>;
    fn fetch(&self, page: &Arc<dyn Page>) -> Option<String>;
    fn assets(&self) -> Vec<Arc<dyn Page>>;
    fn template_assets(&self) -> Vec<TemplateAsset>;
}

#[derive(Clone, Debug)]
pub struct TemplateAsset {
    pub path: Vec<String>,
    pub template_name: String,
    pub metadata: Option<Metadata>,
}

pub struct HandlebarsStage {
    pub name: String,
    pub lookup: Arc<dyn HandlebarsLookup>,
}

impl Stage for HandlebarsStage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing started");
        let lookup_result = self.lookup.lookup(env)?;
        let root_repository = lookup_result.clone_registry();

        // Fetch pages
        let result: Vec<Arc<dyn Page>> = bundle
            .pages()
            .par_iter()
            .filter_map(|p| {
                lookup_result.fetch(p).map(|template_name| {
                    Arc::new(HandlebarsPage {
                        registry: root_repository.clone(),
                        source: Arc::clone(p),
                        template_name,
                    }) as Arc<dyn Page>
                })
            })
            .collect::<Vec<Arc<dyn Page>>>();

        let mut result_bundle = VecBundle { p: result };

        // Append asset pages
        result_bundle.p.append(&mut lookup_result.assets());

        // Append template asset pages
        result_bundle.p.append(
            &mut lookup_result
                .template_assets()
                .iter()
                .map(|t| {
                    Arc::new(HandlebarsTemplatePage {
                        registry: root_repository.clone(),
                        template_asset: t.clone(),
                    }) as Arc<dyn Page>
                })
                .collect(),
        );
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing ended");
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        Ok((
            Arc::new(result_bundle),
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

#[derive(Debug)]
struct HandlebarsTemplatePage {
    registry: handlebars::Handlebars<'static>,
    template_asset: TemplateAsset,
}

impl Page for HandlebarsTemplatePage {
    fn path(&self) -> &[String] {
        &self.template_asset.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.template_asset.metadata.as_ref()
    }

    fn open(&self, _: &PageIndex, _: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let result = self.registry.render(&self.template_asset.template_name, &self.metadata())?;
        Ok(Box::new(Cursor::new(result)))
    }
}

#[derive(Debug)]
struct HandlebarsPage {
    registry: handlebars::Handlebars<'static>,
    source: Arc<dyn Page>,
    template_name: String,
}

#[derive(Serialize)]
pub struct PageData<'a> {
    pub current_metadata: Option<&'a Metadata>,
    pub page: &'a PageIndex,
    pub index: &'a BundleIndex,
}

impl Page for HandlebarsPage {
    fn path(&self) -> &[String] {
        self.source.path()
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.source.metadata()
    }

    fn open(&self, output_page: &PageIndex, output_index: &BundleIndex, env: &Env) -> anyhow::Result<Box<dyn Read>> {
        let mut local_registry = self.registry.clone();
        local_registry.register_helper(
            "content_as_string",
            Box::new(|_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| {
                let mut result = String::new();
                self.source
                    .open(output_page, output_index, env)
                    .map_err(|err| RenderError::new(err.to_string()))?
                    .read_to_string(&mut result)?;
                out.write(&result)?;
                Ok(())
            }),
        );
        let result = (&local_registry).render(
            &self.template_name,
            &PageData {
                current_metadata: self.metadata(),
                page: output_page,
                index: output_index,
            },
        )?;
        Ok(Box::new(Cursor::new(result)))
    }
}

#[derive(Debug)]
pub struct HandlebarsDir {
    pub base_path: PathBuf,
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

impl HandlebarsLookup for HandlebarsDir {
    fn lookup(&self, env: &Env) -> anyhow::Result<Arc<dyn HandlebarsLookupResult>> {
        env.print_vv("HandlebarsDir", &format!("handlebars lookup from dir {}", &self.base_path.to_string_lossy()));
        let mut result = HandlebarsDirResult {
            registry: Handlebars::new(),
            pages: Default::default(),
            template_assets: Default::default(),
            static_assets: Default::default(),
        };

        visit_dirs(&self.base_path, &mut |entry| {
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(&self.base_path)?;
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
                asset_path.push(name[5..name.len() - 3].to_string()); // asset.<name>.hbs -> <name>
                let template_name: String = asset_path.join("/");

                result.registry.register_template_file(&template_name, entry_path)?;
                result.template_assets.push(TemplateAsset {
                    path: asset_path,
                    template_name,
                    metadata: None,
                });
            } else if ext == "hbs" {
                let template_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
                let template_name = template_name.strip_suffix(".hbs").unwrap();
                result.registry.register_template_file(template_name, entry_path)?;
            } else {
                result.static_assets.push(Arc::new(FsPage::new(&self.base_path, entry_path)?));
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

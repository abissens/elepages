use crate::config::Value;
use crate::pages::{BundleIndex, Env, Metadata, Page, PageBundle, PageIndex, VecBundle};
use crate::stages::{BundleQueryHelper, DateFormatHelper, PageContentHelper, PageGeneratorBag, ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
use std::io::{Cursor, Read};
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

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env, _: &Arc<dyn PageGeneratorBag>) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now());
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing started");
        let lookup_result = self.lookup.lookup(env)?;
        let root_repository = lookup_result.clone_registry();

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
                        metadata: t.metadata.clone(),
                        path: t.path.clone(),
                        template_name: t.template_name.clone(),
                    }) as Arc<dyn Page>
                })
                .collect(),
        );
        env.print_vv(&format!("stage {}", self.name()), "handlebars processing ended");
        let end = DateTime::<Utc>::from(SystemTime::now());
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
    path: Vec<String>,
    template_name: String,
    metadata: Option<Metadata>,
}

#[derive(Serialize)]
pub struct TemplateData<'a> {
    pub page: &'a PageIndex,
    pub index: &'a BundleIndex,
}

impl Page for HandlebarsTemplatePage {
    fn path(&self) -> &[String] {
        &self.path
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    fn open(&self, output_page: &PageIndex, output_index: &BundleIndex, _: &Env) -> anyhow::Result<Box<dyn Read>> {
        let mut local_registry = self.registry.clone();
        local_registry.register_helper("bundle_query", Box::new(BundleQueryHelper { output_index }));
        local_registry.register_helper("date_format", Box::new(DateFormatHelper));
        let result = local_registry.render(
            &self.template_name,
            &TemplateData {
                page: output_page,
                index: output_index,
            },
        )?;
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
            "page_content",
            Box::new(PageContentHelper {
                source: &self.source,
                output_page,
                output_index,
                env,
            }),
        );
        local_registry.register_helper("bundle_query", Box::new(BundleQueryHelper { output_index }));
        local_registry.register_helper("date_format", Box::new(DateFormatHelper));
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

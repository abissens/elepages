use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::Arc;

use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use rayon::prelude::*;

use crate::pages::{FsPage, Metadata, Page, PageBundle, VecBundle};
use crate::stages::handlebars_stage::RenderResult::Content;
use crate::stages::stage::Stage;
use crate::utilities::visit_dirs;
use std::any::Any;

pub trait HandlebarsLookup: Sync + Send + Debug {
    fn init_registry(&self, registry: &mut handlebars::Handlebars) -> anyhow::Result<()>;
    fn fetch(&self, page: &Arc<dyn Page>) -> Option<String>;
    fn assets(&self) -> anyhow::Result<Vec<Arc<dyn Page>>>;
    fn as_any(&self) -> &dyn Any {
        panic!("not implemented")
    }
}

pub struct HandlebarsStage {
    pub lookup: Arc<dyn HandlebarsLookup>,
}

impl Stage for HandlebarsStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut registry = handlebars::Handlebars::new();
        self.lookup.init_registry(&mut registry)?;
        let result: Vec<RenderResult> = bundle
            .pages()
            .par_iter()
            .map(|p| {
                return match self.lookup.fetch(p) {
                    None => Ok(RenderResult::Empty),
                    Some(template_name) => {
                        let mut local_registry = registry.clone();
                        local_registry.register_helper(
                            "content_as_string",
                            Box::new(|_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| {
                                let mut result = String::new();
                                p.open().map_err(|err| RenderError::new(err.to_string()))?.read_to_string(&mut result)?;
                                out.write(&result)?;
                                Ok(())
                            }),
                        );
                        let result = (&local_registry).render(&template_name, &p.metadata())?;
                        Ok(Content { value: result, source: Arc::clone(p) })
                    }
                };
            })
            .collect::<anyhow::Result<Vec<RenderResult>>>()?;

        let mut result_bundle = VecBundle { p: vec![] };
        for rr in result {
            match rr {
                RenderResult::Empty => {}
                RenderResult::Content { value, source } => result_bundle.p.push(Arc::new(CursorPage { value, source })),
            }
        }

        result_bundle.p.append(&mut self.lookup.assets()?);

        Ok(Arc::new(result_bundle))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

enum RenderResult {
    Empty,
    Content { value: String, source: Arc<dyn Page> },
}

#[derive(Debug)]
struct CursorPage {
    value: String,
    source: Arc<dyn Page>,
}

impl Page for CursorPage {
    fn path(&self) -> &[String] {
        self.source.path()
    }

    fn metadata(&self) -> Option<&Metadata> {
        self.source.metadata()
    }

    fn open(&self) -> anyhow::Result<Box<dyn Read>> {
        Ok(Box::new(Cursor::new(self.value.clone())))
    }
}

#[derive(Debug)]
pub struct HandlebarsDir {
    pub(crate) prefix_len: usize,
    pub(crate) pages: HashMap<String, PathBuf>,
    pub(crate) template_files: HashMap<String, PathBuf>,
    pub(crate) static_assets: HashMap<String, PathBuf>,
    pub(crate) base_path: PathBuf,
}

impl HandlebarsDir {
    pub fn new(dir: &Path) -> anyhow::Result<Self> {
        let mut result = Self {
            prefix_len: dir.to_string_lossy().len(),
            pages: Default::default(),
            template_files: Default::default(),
            static_assets: Default::default(),
            base_path: dir.into(),
        };
        visit_dirs(dir, &mut |entry| {
            let entry_path = entry.path();
            let rel_path = entry_path.strip_prefix(dir)?;
            let name = entry_path.file_name().map(|e| e.to_string_lossy()).unwrap_or_else(|| "".into());
            let ext = entry_path.extension().map(|e| e.to_string_lossy()).unwrap_or_else(|| "".into());

            let path_name = rel_path.to_string_lossy().replace(MAIN_SEPARATOR, "/");
            let path_name = path_name.strip_suffix(".hbs").unwrap_or(&path_name).into();

            if name.starts_with("page.") && ext == "hbs" {
                // page.hbs or page.<page name>.hbs format
                result.pages.insert(path_name, entry_path);
            } else if ext == "hbs" {
                result.template_files.insert(path_name, entry_path);
            } else {
                result.static_assets.insert(path_name, entry_path);
            }
            Ok(())
        })?;

        Ok(result)
    }
}

fn path_join(v: &[String], i: usize) -> String {
    let mut c: String = v[0..i].join("/");
    if !c.is_empty() {
        c.push('/');
    }
    c
}

impl HandlebarsLookup for HandlebarsDir {
    fn init_registry(&self, registry: &mut Handlebars) -> anyhow::Result<()> {
        for (tpl_name, tpl_path) in &self.template_files {
            registry.register_template_file(tpl_name, tpl_path)?;
        }
        for (tpl_name, tpl_path) in &self.pages {
            registry.register_template_file(tpl_name, tpl_path)?;
        }
        Ok(())
    }

    fn fetch(&self, page: &Arc<dyn Page>) -> Option<String> {
        let page_path = page.path();
        let l = page_path.len();
        let mut c = path_join(page_path, l - 1);
        let mut c1 = c.clone();
        c1.push_str("page.");
        c1.push_str(&page_path[l - 1]);
        if self.pages.contains_key(&c1) {
            return Some(c1);
        }

        c.push_str("page");

        if self.pages.contains_key(&c) {
            return Some(c);
        }

        for i in (0..l - 1).rev() {
            let mut c = path_join(page_path, i);
            c.push_str("page");

            if self.pages.contains_key(&c) {
                return Some(c);
            }
        }

        None
    }

    fn assets(&self) -> anyhow::Result<Vec<Arc<dyn Page>>> {
        let mut result = vec![];

        for sa in self.static_assets.values() {
            result.push(Arc::new(FsPage::new(&self.base_path, sa.clone())?) as Arc<dyn Page>)
        }

        Ok(result)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

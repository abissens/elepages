use crate::pages::{Metadata, Page, PageBundle, VecBundle};
use crate::stages::handlebars_stage::RenderResult::Content;
use crate::stages::stage::Stage;
use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use rayon::prelude::*;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::sync::Arc;

pub trait HandlebarsLookup: Sync + Send + Debug {
    fn init_registry(&self, registry: &mut handlebars::Handlebars);
    fn fetch(&self, page: &Arc<dyn Page>) -> Option<String>;
}

pub struct HandlebarsStage {
    pub lookup: Arc<dyn HandlebarsLookup>,
}

impl Stage for HandlebarsStage {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut registry = handlebars::Handlebars::new();
        self.lookup.init_registry(&mut registry);
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

        Ok(Arc::new(result_bundle))
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

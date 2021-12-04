use crate::pages::{BundleIndex, Env, Page, PageIndex};
use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError};
use std::sync::Arc;

pub struct PageContentHelper<'a> {
    pub source: &'a Arc<dyn Page>,
    pub output_page: &'a PageIndex,
    pub output_index: &'a BundleIndex,
    pub env: &'a Env,
}

impl HelperDef for PageContentHelper<'_> {
    fn call<'reg: 'rc, 'rc>(&self, _: &Helper<'reg, 'rc>, _: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
        let mut result = String::new();
        self.source
            .open(self.output_page, self.output_index, self.env)
            .map_err(|err| RenderError::new(err.to_string()))?
            .read_to_string(&mut result)?;
        out.write(&result)?;
        Ok(())
    }
}

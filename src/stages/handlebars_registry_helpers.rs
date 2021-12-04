use crate::pages::{AndQuery, AuthorQuery, BundleIndex, BundlePagination, BundleQuery, Env, NotQuery, OrQuery, Page, PageIndex, TagQuery};
use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError, ScopedJson};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum BundleQueryValue {
    Tag { tag: String },
    Tags { tags: Vec<String> },
    Author { author: String },
    Authors { authors: Vec<String> },
    And { and: Vec<BundleQueryValue> },
    Or { or: Vec<BundleQueryValue> },
    Not { not: Box<BundleQueryValue> },
}

impl From<&BundleQueryValue> for Box<dyn BundleQuery> {
    fn from(value: &BundleQueryValue) -> Self {
        match value {
            BundleQueryValue::Tag { tag } => Box::new(TagQuery(tag.to_string())),
            BundleQueryValue::Tags { tags } => Box::new(AndQuery(tags.iter().map(|t| Box::new(TagQuery(t.to_string())) as Box<dyn BundleQuery>).collect())),
            BundleQueryValue::Author { author } => Box::new(AuthorQuery(author.to_string())),
            BundleQueryValue::Authors { authors } => Box::new(AndQuery(authors.iter().map(|a| Box::new(AuthorQuery(a.to_string())) as Box<dyn BundleQuery>).collect())),
            BundleQueryValue::And { and } => Box::new(AndQuery(and.iter().map(<Box<dyn BundleQuery>>::from).collect())),
            BundleQueryValue::Or { or } => Box::new(OrQuery(or.iter().map(<Box<dyn BundleQuery>>::from).collect())),
            BundleQueryValue::Not { not } => Box::new(NotQuery(<Box<dyn BundleQuery>>::from(not.as_ref()))),
        }
    }
}

pub struct BundleQueryHelper<'a> {
    pub output_index: &'a BundleIndex,
}

impl HelperDef for BundleQueryHelper<'_> {
    fn call_inner<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let value: BundleQueryValue = serde_yaml::from_str(param).map_err(|err| RenderError::new(err.to_string()))?;
        let bundle_query: Box<dyn BundleQuery> = <Box<dyn BundleQuery>>::from(&value);
        let pages = self.output_index.query(bundle_query.as_ref(), &BundlePagination { skip: None, limit: None });
        Ok(ScopedJson::Derived(serde_json::to_value(pages)?))
    }
}

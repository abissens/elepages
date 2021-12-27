use crate::pages::{BundleIndex, BundlePagination, BundleQuery, Env, Page, PageIndex};
use chrono::{DateTime, NaiveDateTime, Utc};
use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError, ScopedJson};
use std::sync::Arc;
use crate::utilities::uri_friendly_string;

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

pub struct BundleQueryHelper<'a> {
    pub output_index: &'a BundleIndex,
}

impl HelperDef for BundleQueryHelper<'_> {
    fn call_inner<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let param1 = h.param(0).and_then(|v| v.value().as_str());
        let param2 = h.param(1).and_then(|v| v.value().as_str());

        let value: Option<BundleQuery> = match param1 {
            None => None,
            Some(param) => {
                if param.is_empty() {
                    None
                } else {
                    Some(serde_yaml::from_str(param).map_err(|err| RenderError::new(err.to_string()))?)
                }
            }
        };
        let pagination: BundlePagination = match param2 {
            None => BundlePagination { skip: None, limit: None },
            Some(param) => serde_yaml::from_str(param).map_err(|err| RenderError::new(err.to_string()))?,
        };

        let bundle_query: BundleQuery = match value {
            None => BundleQuery::Always,
            Some(value) => value,
        };

        let pages = self.output_index.query(&bundle_query, &pagination);
        Ok(ScopedJson::Derived(serde_json::to_value(pages)?))
    }
}

pub struct DateFormatHelper;

impl HelperDef for DateFormatHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
        let timestamp_param = h.param(0).and_then(|v| v.value().as_i64());
        let format_param = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("%Y-%m-%d");
        if timestamp_param.is_none() {
            return Ok(());
        }
        let naive_dt = NaiveDateTime::from_timestamp(timestamp_param.unwrap(), 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive_dt, Utc);

        out.write(&datetime.format(format_param).to_string())?;
        Ok(())
    }
}


pub struct ForURIHelper;

impl HelperDef for ForURIHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _: &'reg Handlebars<'reg>, _: &'rc Context, _: &mut RenderContext<'reg, 'rc>, out: &mut dyn Output) -> HelperResult {
        let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let converted = uri_friendly_string(param);
        out.write(&converted)?;
        Ok(())
    }
}
use crate::pages::{BundleIndex, Env, Metadata, Page, PageIndex};
use crate::stages::{BundleQueryHelper, DateFormatHelper};
use serde::Serialize;
use std::io::{Cursor, Read};

#[derive(Debug)]
pub(crate) struct HbsAsset {
    pub(crate) registry: handlebars::Handlebars<'static>,
    pub(crate) tpl_name: String,
    pub(crate) path: Vec<String>,
    pub(crate) metadata: Option<Metadata>,
}

impl Page for HbsAsset {
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
            &self.tpl_name,
            &AssetData {
                page: output_page,
                index: output_index,
            },
        )?;
        Ok(Box::new(Cursor::new(result)))
    }
}

#[derive(Serialize)]
pub struct AssetData<'a> {
    pub page: &'a PageIndex,
    pub index: &'a BundleIndex,
}

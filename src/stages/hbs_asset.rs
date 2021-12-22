use crate::pages::{BundleIndex, Env, Metadata, Page, PageIndex};
use crate::stages::{BundleQueryHelper, DateFormatHelper};
use serde::Serialize;
use std::io::{Cursor, Read};

#[derive(Debug, Serialize)]
pub(crate) struct HbsAssetSelection {
    pub(crate) pages: Vec<PageIndex>,
    pub(crate) index: usize,
    pub(crate) last: usize,
    pub(crate) limit: usize,
    pub(crate) size: Option<usize>,
    pub(crate) tag: Option<String>,
    pub(crate) author: Option<String>,
}

#[derive(Debug)]
pub(crate) struct HbsAsset {
    pub(crate) registry: handlebars::Handlebars<'static>,
    pub(crate) tpl_name: String,
    pub(crate) path: Vec<String>,
    pub(crate) metadata: Option<Metadata>,
    pub(crate) selection: Option<HbsAssetSelection>,
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
                selection: if let Some(s) = &self.selection { Some(s) } else { None },
            },
        )?;
        Ok(Box::new(Cursor::new(result)))
    }
}

#[derive(Serialize)]
pub(crate) struct AssetData<'a> {
    pub(crate) page: &'a PageIndex,
    pub(crate) index: &'a BundleIndex,
    pub(crate) selection: Option<&'a HbsAssetSelection>,
}

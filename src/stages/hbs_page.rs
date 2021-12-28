use crate::pages::{BundleIndex, Env, Metadata, Page, PageIndex};
use crate::stages::{BundleArchiveHelper, BundleQueryHelper, DateFormatHelper, ForUriHelper, PageContentHelper};
use serde::Serialize;
use std::io::{Cursor, Read};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct HbsPage {
    pub(crate) registry: handlebars::Handlebars<'static>,
    pub(crate) source: Arc<dyn Page>,
    pub(crate) tpl_name: String,
}

impl Page for HbsPage {
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
        local_registry.register_helper("bundle_archive_query", Box::new(BundleArchiveHelper { output_index }));
        local_registry.register_helper("date_format", Box::new(DateFormatHelper));
        local_registry.register_helper("uri_string", Box::new(ForUriHelper));
        let result = (&local_registry).render(
            &self.tpl_name,
            &PageData {
                current_metadata: self.metadata(),
                page: output_page,
                index: output_index,
            },
        )?;
        Ok(Box::new(Cursor::new(result)))
    }
}

#[derive(Serialize)]
pub struct PageData<'a> {
    pub current_metadata: Option<&'a Metadata>,
    pub page: &'a PageIndex,
    pub index: &'a BundleIndex,
}

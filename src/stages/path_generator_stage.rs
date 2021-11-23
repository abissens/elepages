use crate::config::FromValue;
use crate::pages::{ArcPage, Env, MetadataIndex, PageBundle, VecBundle};
use crate::stages::{ProcessingResult, Stage};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;

pub struct PathGenerator {
    name: String,
}

impl PathGenerator {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
impl Stage for PathGenerator {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&self, bundle: &Arc<dyn PageBundle>, env: &Env) -> anyhow::Result<(Arc<dyn PageBundle>, ProcessingResult)> {
        let start = DateTime::<Utc>::from(SystemTime::now()).timestamp();
        let mut result = VecBundle { p: vec![] };
        env.print_vv(&format!("stage {}", self.name()), "path generation");

        let mut candidates = vec![];
        for page in bundle.pages() {
            if let Some(Some(path_value)) = page.metadata().map(|m| m.data.get("path")) {
                if let Ok(path) = String::from_value(path_value.clone()) {
                    candidates.push((page, path));
                    continue;
                }
            }
            result.p.push(Arc::clone(page));
        }

        if !candidates.is_empty() {
            let mut registry = Handlebars::new();
            registry.set_strict_mode(true);
            for (page, path) in candidates {
                let metadata_index = page.metadata().map(MetadataIndex::from);
                let rendered_path = registry.render_template(&path, &metadata_index)?.split('/').map(|s| s.to_string()).collect();
                result.p.push(page.change_path(rendered_path));
            }
        }

        env.print_vv(&format!("stage {}", self.name()), "path generation ended");
        let end = DateTime::<Utc>::from(SystemTime::now()).timestamp();

        Ok((
            Arc::new(result),
            ProcessingResult {
                stage_name: self.name(),
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

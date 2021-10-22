use crate::pages::{Metadata, Page, PageBundle, PageProxy, VecBundle};
use crate::stages::metadata_tree::MetadataTree;
use crate::stages::stage::Stage;
use rayon::prelude::*;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::option::Option::Some;
use std::sync::Arc;

pub trait ShadowLoader: Send + Sync {
    fn load(&self, page: Arc<dyn Page>) -> anyhow::Result<Metadata>;
}

pub struct ShadowPages {
    loaders: HashMap<String, Arc<dyn ShadowLoader>>,
}

impl Stage for ShadowPages {
    fn process(&self, bundle: &Arc<dyn PageBundle>) -> anyhow::Result<Arc<dyn PageBundle>> {
        let mut vec_bundle = VecBundle { p: vec![] };

        let mut metadata_candidates = vec![];
        let mut all_paths = HashSet::new();

        // select metadata candidates
        for page in bundle.pages() {
            let path = page.path();
            for (ext, loader) in &self.loaders {
                if path[path.len() - 1].ends_with(ext) {
                    metadata_candidates.push(MetadataCandidate {
                        path: page
                            .path()
                            .iter()
                            .enumerate()
                            .map(|(pos, p)| {
                                if pos == path.len() - 1 {
                                    return p.strip_suffix(ext).unwrap().to_string();
                                }
                                p.to_string()
                            })
                            .collect(),
                        page,
                        loader,
                    })
                }
            }
            for i in 0..path.len() {
                all_paths.insert(&path[..i + 1]);
            }
        }

        // retain only shadow pages
        metadata_candidates.retain(|c| all_paths.contains(c.path.as_slice()));

        let loaded_metadata_vec: Vec<LoadedMetadata> = metadata_candidates
            .par_iter()
            .map(|metadata_candidate: &MetadataCandidate| {
                let c_page = Arc::clone(metadata_candidate.page);
                let path = metadata_candidate.path.clone();
                metadata_candidate.loader.load(c_page).map(|metadata| LoadedMetadata { path, metadata })
            })
            .collect::<anyhow::Result<Vec<LoadedMetadata>>>()?;

        // feed metadata tree
        let mut metadata_tree = MetadataTree::Root { sub: HashMap::new() };

        for loaded_metadata in loaded_metadata_vec {
            metadata_tree.push(&loaded_metadata.path, loaded_metadata.metadata)?
        }

        let metadata_pages_set = metadata_candidates.iter().map(|c| c.page.path().to_vec()).collect::<HashSet<Vec<String>>>();

        // push non metadata pages to result bundle
        for page in bundle.pages() {
            if !metadata_pages_set.contains(page.path()) {
                // get path metadata
                let mut metadata_vec = vec![];
                metadata_tree.get_metadata_from_path(page.path(), &mut metadata_vec);
                if metadata_vec.is_empty() {
                    vec_bundle.p.push(Arc::clone(page));
                    continue;
                }
                let mut current_metadata: Metadata;
                if let Some(page_metadata) = page.metadata() {
                    // page has already metadata
                    current_metadata = page_metadata.clone();
                } else if metadata_vec.len() == page.path().len() {
                    // dedicated metadata for current page
                    current_metadata = metadata_vec
                        .pop()
                        .unwrap()
                        .metadata
                        .unwrap_or(&Metadata {
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                        })
                        .clone();
                } else {
                    // default metadata
                    current_metadata = Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                    }
                }

                while let Some(metadata_node) = metadata_vec.pop() {
                    if let Some(m) = metadata_node.metadata {
                        current_metadata = current_metadata.merge(m)?;
                    }
                }
                vec_bundle.p.push(Arc::new(PageProxy {
                    new_path: None,
                    new_metadata: Some(current_metadata),
                    inner: Arc::clone(page),
                }));
            }
        }

        Ok(Arc::new(vec_bundle))
    }
}

impl ShadowPages {
    pub fn new(loaders: HashMap<String, Arc<dyn ShadowLoader>>) -> Self {
        ShadowPages { loaders }
    }

    pub fn default() -> Self {
        ShadowPages {
            loaders: IntoIter::new([
                (".json".to_string(), Arc::new(JsonShadowLoader()) as Arc<dyn ShadowLoader>),
                (".yaml".to_string(), Arc::new(YamlShadowLoader()) as Arc<dyn ShadowLoader>),
            ])
            .collect(),
        }
    }
}

struct JsonShadowLoader();
struct YamlShadowLoader();

impl ShadowLoader for JsonShadowLoader {
    fn load(&self, page: Arc<dyn Page>) -> anyhow::Result<Metadata> {
        Ok(serde_json::from_reader(page.open()?)?)
    }
}

impl ShadowLoader for YamlShadowLoader {
    fn load(&self, page: Arc<dyn Page>) -> anyhow::Result<Metadata> {
        Ok(serde_yaml::from_reader(page.open()?)?)
    }
}

struct MetadataCandidate<'a> {
    path: Vec<String>,
    page: &'a Arc<dyn Page>,
    loader: &'a Arc<dyn ShadowLoader>,
}

struct LoadedMetadata {
    path: Vec<String>,
    metadata: Metadata,
}

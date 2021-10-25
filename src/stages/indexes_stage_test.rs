#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, Metadata, PageBundle, VecBundle};
    use crate::stages::indexes_stage::IndexStage;
    use crate::stages::stage::Stage;
    use serde::{Deserialize, Serialize};
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::hash::{Hash, Hasher};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn generate_index_pages_from_single_page_bundle() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![Arc::new(TestPage {
                path: vec!["dir".to_string(), "f1".to_string()],
                metadata: Some(Metadata {
                    title: Some("f1 title".to_string()),
                    summary: Some("f1 summary".to_string()),
                    authors: HashSet::from_iter(IntoIter::new([Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default(),
                    }])),
                    tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                }),
                content: String::new(),
            })],
        });

        let index_stage = IndexStage {};
        let result_bundle = index_stage.process(&vec_bundle).unwrap();

        assert_eq!(
            IndexPages::from_bundle(&result_bundle),
            IndexPages {
                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    ("t1".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                    ("t2".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                    ("t3".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                ])),
                pages_by_author: HashMap::from_iter(IntoIter::new([(
                    "f1 author".to_string(),
                    HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))
                ),])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                all_pages: HashSet::from_iter(IntoIter::new([TestPageIndex {
                    path: vec!["dir".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: Some("f1 title".to_string()),
                        summary: Some("f1 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([Author {
                            name: "f1 author".to_string(),
                            contacts: Default::default()
                        }])),
                        tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()]))
                    })
                }])),
                all_authors: HashSet::from_iter(IntoIter::new([Author {
                    name: "f1 author".to_string(),
                    contacts: Default::default()
                }]))
            }
        );
    }

    #[test]
    fn generate_index_pages_from_multi_page_bundle() {
        let vec_bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f1".to_string()],
                    metadata: Some(Metadata {
                        title: Some("f1 title".to_string()),
                        summary: Some("f1 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([Author {
                            name: "f1 author".to_string(),
                            contacts: Default::default(),
                        }])),
                        tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2".to_string()],
                    metadata: None,
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f3".to_string()],
                    metadata: Some(Metadata {
                        title: Some("f3 title".to_string()),
                        summary: Some("f3 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([
                            Author {
                                name: "f3 author 1".to_string(),
                                contacts: Default::default(),
                            },
                            Author {
                                name: "f3 author 2".to_string(),
                                contacts: Default::default(),
                            },
                        ])),
                        tags: HashSet::from_iter(IntoIter::new(["t3".to_string(), "t4".to_string()])),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some("f4 title".to_string()),
                        summary: Some("f4 summary".to_string()),
                        authors: HashSet::from_iter(IntoIter::new([Author {
                            name: "f3 author 1".to_string(),
                            contacts: Default::default(),
                        }])),
                        tags: HashSet::default(),
                    }),
                    content: String::new(),
                }),
            ],
        });

        let index_stage = IndexStage {};
        let result_bundle = index_stage.process(&vec_bundle).unwrap();
        let index_pages = IndexPages::from_bundle(&result_bundle);
        assert_eq!(
            index_pages,
            IndexPages {
                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    ("t1".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                    ("t2".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                    ("t3".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()], vec!["f3".to_string()]]))),
                    ("t4".to_string(), HashSet::from_iter(IntoIter::new([vec!["f3".to_string()]]))),
                ])),
                pages_by_author: HashMap::from_iter(IntoIter::new([
                    ("f1 author".to_string(), HashSet::from_iter(IntoIter::new([vec!["dir".to_string(), "f1".to_string()]]))),
                    ("f3 author 1".to_string(), HashSet::from_iter(IntoIter::new([vec!["f3".to_string()], vec!["f4".to_string()]]))),
                    ("f3 author 2".to_string(), HashSet::from_iter(IntoIter::new([vec!["f3".to_string()]]))),
                ])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string(), "t4".to_string()])),
                all_pages: HashSet::from_iter(IntoIter::new([
                    TestPageIndex {
                        path: vec!["dir".to_string(), "f1".to_string()],
                        metadata: Some(Metadata {
                            title: Some("f1 title".to_string()),
                            summary: Some("f1 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new([Author {
                                name: "f1 author".to_string(),
                                contacts: Default::default()
                            }])),
                            tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()]))
                        })
                    },
                    TestPageIndex {
                        path: vec!["f2".to_string()],
                        metadata: None
                    },
                    TestPageIndex {
                        path: vec!["f3".to_string()],
                        metadata: Some(Metadata {
                            title: Some("f3 title".to_string()),
                            summary: Some("f3 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new([
                                Author {
                                    name: "f3 author 1".to_string(),
                                    contacts: Default::default()
                                },
                                Author {
                                    name: "f3 author 2".to_string(),
                                    contacts: Default::default()
                                }
                            ])),
                            tags: HashSet::from_iter(IntoIter::new(["t3".to_string(), "t4".to_string()]))
                        })
                    },
                    TestPageIndex {
                        path: vec!["f4".to_string()],
                        metadata: Some(Metadata {
                            title: Some("f4 title".to_string()),
                            summary: Some("f4 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new([Author {
                                name: "f3 author 1".to_string(),
                                contacts: Default::default()
                            }])),
                            tags: HashSet::default(),
                        })
                    }
                ])),
                all_authors: HashSet::from_iter(IntoIter::new([
                    Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default()
                    },
                    Author {
                        name: "f3 author 1".to_string(),
                        contacts: Default::default()
                    },
                    Author {
                        name: "f3 author 2".to_string(),
                        contacts: Default::default()
                    }
                ]))
            }
        );
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPageIndex {
        path: Vec<String>,
        metadata: Option<Metadata>,
    }

    impl Hash for TestPageIndex {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.path.hash(state)
        }
    }
    impl Eq for TestPageIndex {}

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct IndexPages {
        pages_by_tag: HashMap<String, HashSet<Vec<String>>>,
        pages_by_author: HashMap<String, HashSet<Vec<String>>>,
        all_tags: HashSet<String>,
        all_pages: HashSet<TestPageIndex>,
        all_authors: HashSet<Author>,
    }

    impl IndexPages {
        fn from_bundle(bundle: &Arc<dyn PageBundle>) -> Self {
            let mut pages_by_tag: HashMap<String, HashSet<Vec<String>>> = Default::default();
            let mut pages_by_author: HashMap<String, HashSet<Vec<String>>> = Default::default();
            let mut all_tags: HashSet<String> = Default::default();
            let mut all_pages: HashSet<TestPageIndex> = Default::default();
            let mut all_authors: HashSet<Author> = Default::default();
            for page in bundle.pages() {
                match page.path().join("/").as_str() {
                    "pages_by_tag.json" => pages_by_tag = serde_json::from_reader(page.open().unwrap()).unwrap(),
                    "pages_by_author.json" => pages_by_author = serde_json::from_reader(page.open().unwrap()).unwrap(),
                    "all_tags.json" => all_tags = serde_json::from_reader(page.open().unwrap()).unwrap(),
                    "all_pages.json" => all_pages = serde_json::from_reader(page.open().unwrap()).unwrap(),
                    "all_authors.json" => all_authors = serde_json::from_reader(page.open().unwrap()).unwrap(),
                    _ => panic!("should not have other file"),
                }
            }

            Self {
                pages_by_tag,
                pages_by_author,
                all_tags,
                all_pages,
                all_authors,
            }
        }
    }
}

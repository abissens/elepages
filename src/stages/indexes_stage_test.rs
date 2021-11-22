#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, BundleIndex, Env, Metadata, PageBundle, PageIndex, VecBundle};
    use crate::stages::indexes_stage::IndexStage;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{ComposeStage, ComposeUnit};
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
                    title: Some(Arc::new("f1 title".to_string())),
                    summary: Some(Arc::new("f1 summary".to_string())),
                    authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                        name: "f1 author".to_string(),
                        contacts: Default::default(),
                    })])),
                    tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                    publishing_date: None,
                    last_edit_date: None,
                    data: HashMap::from_iter(IntoIter::new([
                        ("a".to_string(), Value::String("a".to_string())),
                        (
                            "b".to_string(),
                            Value::Vec(vec![Value::String("1".to_string()), Value::String("2".to_string()), Value::String("3".to_string())]),
                        ),
                        ("c".to_string(), Value::I32(10)),
                    ])),
                }),
                content: String::new(),
            })],
        });

        let index_stage = ComposeStage {
            name: "compose".to_string(),
            parallel: true,
            units: vec![Arc::new(ComposeUnit::CreateNewSet(Arc::new(IndexStage { name: "index stage".to_string() })))],
        };

        let result_bundle = index_stage.process(&vec_bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "index stage".to_string(),
                    sub_results: vec![]
                }]
            }
        );
        assert_eq!(
            IndexPages::from_bundle(&result_bundle.0),
            IndexPages {
                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    (
                        "t1".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                    (
                        "t2".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                    (
                        "t3".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                ])),
                pages_by_author: HashMap::from_iter(IntoIter::new([(
                    "f1 author".to_string(),
                    HashSet::from_iter(IntoIter::new([TestPageRef {
                        path: vec!["dir".to_string(), "f1".to_string()]
                    }]))
                ),])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                all_pages: HashSet::from_iter(IntoIter::new([
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        },
                        metadata: Some(TestPageIndexMetadata {
                            title: Some("f1 title".to_string()),
                            summary: Some("f1 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::from_iter(IntoIter::new([
                                ("a".to_string(), Value::String("a".to_string())),
                                (
                                    "b".to_string(),
                                    Value::Vec(vec![Value::String("1".to_string()), Value::String("2".to_string()), Value::String("3".to_string())])
                                ),
                                ("c".to_string(), Value::I32(10)),
                            ])),
                        })
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_pages.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_authors.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_tags.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["pages_by_tag.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["pages_by_author.json".to_string()]
                        },
                        metadata: None
                    }
                ])),
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
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: Some(Arc::new("f1 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f1 author".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
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
                        title: Some(Arc::new("f3 title".to_string())),
                        summary: Some(Arc::new("f3 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([
                            Arc::new(Author {
                                name: "f3 author 1".to_string(),
                                contacts: Default::default(),
                            }),
                            Arc::new(Author {
                                name: "f3 author 2".to_string(),
                                contacts: Default::default(),
                            }),
                        ])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t3".to_string()), Arc::new("t4".to_string())])),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: Some(Arc::new("f4 summary".to_string())),
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "f3 author 1".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: String::new(),
                }),
            ],
        });

        let index_stage = ComposeStage {
            name: "compose".to_string(),
            parallel: true,
            units: vec![Arc::new(ComposeUnit::CreateNewSet(Arc::new(IndexStage { name: "index stage".to_string() })))],
        };
        let result_bundle = index_stage.process(&vec_bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "index stage".to_string(),
                    sub_results: vec![]
                }]
            }
        );
        let index_pages = IndexPages::from_bundle(&result_bundle.0);
        assert_eq!(
            index_pages,
            IndexPages {
                pages_by_tag: HashMap::from_iter(IntoIter::new([
                    (
                        "t1".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                    (
                        "t2".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                    (
                        "t3".to_string(),
                        HashSet::from_iter(IntoIter::new([
                            TestPageRef {
                                path: vec!["dir".to_string(), "f1".to_string()]
                            },
                            TestPageRef { path: vec!["f3".to_string()] }
                        ]))
                    ),
                    ("t4".to_string(), HashSet::from_iter(IntoIter::new([TestPageRef { path: vec!["f3".to_string()] }]))),
                ])),
                pages_by_author: HashMap::from_iter(IntoIter::new([
                    (
                        "f1 author".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        }]))
                    ),
                    (
                        "f3 author 1".to_string(),
                        HashSet::from_iter(IntoIter::new([TestPageRef { path: vec!["f3".to_string()] }, TestPageRef { path: vec!["f4".to_string()] }]))
                    ),
                    ("f3 author 2".to_string(), HashSet::from_iter(IntoIter::new([TestPageRef { path: vec!["f3".to_string()] }]))),
                ])),
                all_tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string(), "t4".to_string()])),
                all_pages: HashSet::from_iter(IntoIter::new([
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["dir".to_string(), "f1".to_string()]
                        },
                        metadata: Some(TestPageIndexMetadata {
                            title: Some("f1 title".to_string()),
                            summary: Some("f1 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f1 author".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t1".to_string(), "t2".to_string(), "t3".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        })
                    },
                    TestPageIndex {
                        page_ref: TestPageRef { path: vec!["f2".to_string()] },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef { path: vec!["f3".to_string()] },
                        metadata: Some(TestPageIndexMetadata {
                            title: Some("f3 title".to_string()),
                            summary: Some("f3 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string(), "f3 author 2".to_string()])),
                            tags: HashSet::from_iter(IntoIter::new(["t3".to_string(), "t4".to_string()])),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        })
                    },
                    TestPageIndex {
                        page_ref: TestPageRef { path: vec!["f4".to_string()] },
                        metadata: Some(TestPageIndexMetadata {
                            title: Some("f4 title".to_string()),
                            summary: Some("f4 summary".to_string()),
                            authors: HashSet::from_iter(IntoIter::new(["f3 author 1".to_string()])),
                            tags: HashSet::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        })
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_pages.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_authors.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["all_tags.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["pages_by_tag.json".to_string()]
                        },
                        metadata: None
                    },
                    TestPageIndex {
                        page_ref: TestPageRef {
                            path: vec!["pages_by_author.json".to_string()]
                        },
                        metadata: None
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
        page_ref: TestPageRef,
        metadata: Option<TestPageIndexMetadata>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPageIndexMetadata {
        title: Option<String>,
        summary: Option<String>,
        #[serde(default = "HashSet::default")]
        authors: HashSet<String>,
        #[serde(default = "HashSet::default")]
        tags: HashSet<String>,
        #[serde(alias = "publishingDate")]
        publishing_date: Option<i64>,
        #[serde(alias = "lastEditDate")]
        last_edit_date: Option<i64>,
        #[serde(default = "HashMap::default")]
        data: HashMap<String, Value>,
    }

    impl Hash for TestPageIndex {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.page_ref.hash(state)
        }
    }
    impl Eq for TestPageIndex {}

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct IndexPages {
        pages_by_tag: HashMap<String, HashSet<TestPageRef>>,
        pages_by_author: HashMap<String, HashSet<TestPageRef>>,
        all_tags: HashSet<String>,
        all_pages: HashSet<TestPageIndex>,
        all_authors: HashSet<Author>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct TestPageRef {
        pub path: Vec<String>,
    }

    impl IndexPages {
        fn from_bundle(bundle: &Arc<dyn PageBundle>) -> Self {
            let mut pages_by_tag: HashMap<String, HashSet<TestPageRef>> = Default::default();
            let mut pages_by_author: HashMap<String, HashSet<TestPageRef>> = Default::default();
            let mut all_tags: HashSet<String> = Default::default();
            let mut all_pages: HashSet<TestPageIndex> = Default::default();
            let mut all_authors: HashSet<Author> = Default::default();
            let output_index = BundleIndex::from(bundle);
            println!("{:?}", output_index);
            for page in bundle.pages() {
                let page_index = PageIndex::from(page);
                match page.path().join("/").as_str() {
                    "pages_by_tag.json" => pages_by_tag = serde_json::from_reader(page.open(&page_index, &output_index, &Env::test()).unwrap()).unwrap(),
                    "pages_by_author.json" => pages_by_author = serde_json::from_reader(page.open(&page_index, &output_index, &Env::test()).unwrap()).unwrap(),
                    "all_tags.json" => all_tags = serde_json::from_reader(page.open(&page_index, &output_index, &Env::test()).unwrap()).unwrap(),
                    "all_pages.json" => all_pages = serde_json::from_reader(page.open(&page_index, &output_index, &Env::test()).unwrap()).unwrap(),
                    "all_authors.json" => all_authors = serde_json::from_reader(page.open(&page_index, &output_index, &Env::test()).unwrap()).unwrap(),
                    file => {
                        println!("extra file : {}", file)
                    }
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

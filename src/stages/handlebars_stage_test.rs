#[cfg(test)]
mod tests {
    use crate::config::Value;
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, BundleIndex, Env, Metadata, Page, PageBundle, PageIndex, VecBundle};
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{AppendStage, HandlebarsDir, HandlebarsLookup, HandlebarsLookupResult, HandlebarsStage, Stage, TemplateAsset};
    use indoc::indoc;
    use rustassert::fs::{FileNode, TmpTestFolder};
    use std::array::IntoIter;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn apply_handle_bar_template_to_bundle() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });
        let mut registry = handlebars::Handlebars::new();
        registry.register_template_string("tpl_1", "TPL 1 : {{page.metadata.title}} \n {{page_content}}").unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(NewHandlebarsLookupTest {
                registry,
                fetch: Some("tpl_1".to_string()),
                assets: vec![],
                template_assets: vec![],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL 1 : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn ignore_raw_content() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true))])),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f2 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(false))])),
                    }),
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });
        let mut registry = handlebars::Handlebars::new();
        registry.register_template_string("tpl_1", "TPL 1 : {{page.metadata.title}} \n {{page_content}}").unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(NewHandlebarsLookupTest {
                registry,
                fetch: Some("tpl_1".to_string()),
                assets: vec![],
                template_assets: vec![],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true))]))
                    }),
                    content: "content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f2 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(false))]))
                    }),
                    content: "TPL 1 : f2 title \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn apply_handle_bar_template_to_bundle_and_insert_static_asset_pages() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });

        let mut registry = handlebars::Handlebars::new();
        registry.register_template_string("tpl_1", "TPL 1 : {{page.metadata.title}} \n {{page_content}}").unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(NewHandlebarsLookupTest {
                registry,
                fetch: Some("tpl_1".to_string()),
                assets: vec![
                    Arc::new(TestPage {
                        path: vec!["a".to_string()],
                        metadata: None,
                        content: "a content".to_string(),
                    }),
                    Arc::new(TestPage {
                        path: vec!["b".to_string()],
                        metadata: Some(Metadata {
                            title: Some(Arc::new("b title".to_string())),
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        }),
                        content: "b content".to_string(),
                    }),
                ],
                template_assets: vec![],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string()],
                    metadata: None,
                    content: "a content".to_string()
                },
                TestPage {
                    path: vec!["b".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("b title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "b content".to_string()
                },
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL 1 : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn apply_handle_bar_template_to_bundle_and_insert_template_asset_pages() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });

        let mut registry = handlebars::Handlebars::new();
        registry.register_template_string("tpl_1", "TPL 1 : {{page.metadata.title}} \n {{page_content}}").unwrap();
        registry.register_template_string("tpl_2", "TPL 2 : TPL 2 Content").unwrap();
        registry.register_template_string("tpl_3", "TPL 3 : TPL 3 Content").unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(NewHandlebarsLookupTest {
                registry,
                fetch: Some("tpl_1".to_string()),
                assets: vec![],
                template_assets: vec![
                    TemplateAsset {
                        path: vec!["a".to_string()],
                        template_name: "tpl_2".to_string(),
                        metadata: None,
                    },
                    TemplateAsset {
                        path: vec!["b".to_string()],
                        template_name: "tpl_3".to_string(),
                        metadata: Some(Metadata {
                            title: Some(Arc::new("b title".to_string())),
                            summary: None,
                            authors: Default::default(),
                            tags: Default::default(),
                            publishing_date: None,
                            last_edit_date: None,
                            data: HashMap::default(),
                        }),
                    },
                ],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["a".to_string()],
                    metadata: None,
                    content: "TPL 2 : TPL 2 Content".to_string()
                },
                TestPage {
                    path: vec!["b".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("b title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL 3 : TPL 3 Content".to_string()
                },
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL 1 : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL 1 :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn use_handlebars_dir_single_page_template_lookup_loader() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![FileNode::File {
                    name: "page.hbs".to_string(),
                    content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                    open_options: None,
                }],
            })
            .unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(HandlebarsDir {
                base_path: test_folder.get_path().join("templates"),
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL root : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn use_handlebars_dir_sub_page_template_lookup_loader() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.hbs".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![FileNode::File {
                            name: "page.hbs".to_string(),
                            content: "TPL dir : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                            open_options: None,
                        }],
                    },
                ],
            })
            .unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(HandlebarsDir {
                base_path: test_folder.get_path().join("templates"),
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL dir :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL root : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn use_handlebars_page_name_template_lookup_loader() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: None,
                    content: "content 4".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.hbs".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "TPL f4 : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                        ],
                    },
                ],
            })
            .unwrap();

        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(HandlebarsDir {
                base_path: test_folder.get_path().join("templates"),
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL dir :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: None,
                    content: "TPL f4 :  \n content 4".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL root : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn use_handlebars_template_partial_lookup_loader() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 4".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f5.html".to_string()],
                    metadata: None,
                    content: "content 5".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "page.hbs".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "base.hbs".to_string(),
                        content: "TPL base : {{page.metadata.title}} \n {{> page}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "base.hbs".to_string(),
                                content: "TPL base 2 : {{page.metadata.title}} \n {{> page}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "{{#> base}}{{#*inline \"page\"}}inner: {{page_content}}{{/inline}} {{/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f5.html.hbs".to_string(),
                                content: "{{#> dir/base}}{{#*inline \"page\"}}inner: {{page_content}}{{/inline}} {{/dir/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                        ],
                    },
                ],
            })
            .unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(HandlebarsDir {
                base_path: test_folder.get_path().join("templates"),
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL dir :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL base : f4 title \n inner: content 4".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f5.html".to_string()],
                    metadata: None,
                    content: "TPL base 2 :  \n inner: content 5".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL root : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn use_handlebars_template_generates_static_assets() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 1".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "content 2".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "content 3".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "content 4".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f5.html".to_string()],
                    metadata: None,
                    content: "content 5".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![
                    FileNode::File {
                        name: "index.html".to_string(),
                        content: "test index".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "page.hbs".to_string(),
                        content: "TPL root : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "base.hbs".to_string(),
                        content: "TPL base : {{page.metadata.title}} \n {{> page}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "style.css".to_string(),
                                content: "test css".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "base.hbs".to_string(),
                                content: "TPL base 2 : {{page.metadata.title}} \n {{> page}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{page.metadata.title}} \n {{page_content}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "{{#> base}}{{#*inline \"page\"}}inner: {{page_content}}{{/inline}} {{/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f5.html.hbs".to_string(),
                                content: "{{#> dir/base}}{{#*inline \"page\"}}inner: {{page_content}}{{/inline}} {{/dir/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::Dir {
                                name: "dir2".to_string(),
                                sub: vec![FileNode::File {
                                    name: "main.js".to_string(),
                                    content: "test JS".as_bytes().to_vec(),
                                    open_options: None,
                                }],
                            },
                        ],
                    },
                ],
            })
            .unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(HandlebarsDir {
                base_path: test_folder.get_path().join("templates"),
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "hb stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "dir2".to_string(), "main.js".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))])),
                    }),
                    content: "test JS".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f3.html".to_string()],
                    metadata: None,
                    content: "TPL dir :  \n content 3".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f4.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL base : f4 title \n inner: content 4".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "f5.html".to_string()],
                    metadata: None,
                    content: "TPL base 2 :  \n inner: content 5".to_string(),
                },
                TestPage {
                    path: vec!["dir".to_string(), "style.css".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))])),
                    }),
                    content: "test css".to_string(),
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "TPL root : f1 title \n content 1".to_string(),
                },
                TestPage {
                    path: vec!["f2.htm".to_string()],
                    metadata: None,
                    content: "TPL root :  \n content 2".to_string(),
                },
                TestPage {
                    path: vec!["index.html".to_string()],
                    metadata: Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))])),
                    }),
                    content: "test index".to_string(),
                },
            ]
        );
    }

    #[test]
    fn apply_bundle_query_helper() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(100),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f3".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f3 title".to_string())),
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                            name: "a1".to_string(),
                            contacts: Default::default(),
                        })])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: Some(200),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: HashSet::from_iter(IntoIter::new([
                            Arc::new(Author {
                                name: "a1".to_string(),
                                contacts: Default::default(),
                            }),
                            Arc::new(Author {
                                name: "a2".to_string(),
                                contacts: Default::default(),
                            }),
                        ])),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(300),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f5".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f5 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t4".to_string())])),
                        publishing_date: Some(400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir1".to_string(), "dir2".to_string(), "f6".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f6 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![FileNode::File {
                    name: "asset.index.html.hbs".to_string(),
                    content: indoc! {"
                            {{#each (bundle_query \"{tag: t1}\") }}
                            <h1>{{this.metadata.title}}</h1>
                            {{/each}}
                            {{#each (bundle_query \"{tags: [t1, t2]}\") }}
                            <h2>{{this.metadata.title}}</h2>
                            {{/each}}
                            {{#each (bundle_query \"{author: 'a1'}\") }}
                            <h3>{{this.metadata.title}}</h3>
                            {{/each}}
                            {{#each (bundle_query \"and: [{author: a1}, {tag: t3}]\") }}
                            <h4>{{this.metadata.title}}</h4>
                            {{/each}}
                            {{#each (bundle_query \"and: [{author: a1}, {tag: t4}]\") }}
                            <h4>{{this.metadata.title}}</h4>
                            {{/each}}
                            {{#each (bundle_query \"{path: dir1/**}\") }}
                            <h5>{{this.metadata.title}}</h5>
                            {{/each}}"}
                    .as_bytes()
                    .to_vec(),
                    open_options: None,
                }],
            })
            .unwrap();

        let stages = AppendStage {
            name: "append".to_string(),
            inner: Arc::new(HandlebarsStage {
                name: "hb stage".to_string(),
                lookup: Arc::new(HandlebarsDir {
                    base_path: test_folder.get_path().join("templates"),
                }),
            }),
        };

        let result_bundle = stages.process(&bundle, &Env::test()).unwrap();

        let index_page = result_bundle
            .0
            .pages()
            .iter()
            .find(|p| p.path() == vec!["index.html"].as_slice())
            .map(|p| {
                let mut content: String = "".to_string();
                assert_eq!(
                    p.metadata(),
                    Some(Metadata {
                        title: None,
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: None,
                        last_edit_date: None,
                        data: HashMap::from_iter(IntoIter::new([("isRaw".to_string(), Value::Bool(true)), ("isHidden".to_string(), Value::Bool(true))])),
                    })
                    .as_ref()
                );

                p.open(&PageIndex::from(p), &BundleIndex::from(&result_bundle.0), &Env::test())
                    .unwrap()
                    .read_to_string(&mut content)
                    .unwrap();
                content
            })
            .unwrap();
        assert_eq!(
            index_page,
            indoc! {"
                <h1>f5 title</h1>
                <h1>f4 title</h1>
                <h1>f3 title</h1>
                <h2>f4 title</h2>
                <h2>f3 title</h2>
                <h3>f4 title</h3>
                <h3>f3 title</h3>
                <h4>f3 title</h4>
                <h5>f6 title</h5>
                "
            }
        );
    }

    #[test]
    fn apply_bundle_query_helper_pagination() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f1 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
                        publishing_date: Some(100),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f3".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f3 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                        publishing_date: Some(200),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f4".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f4 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                        publishing_date: Some(300),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f5".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f5 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t4".to_string())])),
                        publishing_date: Some(400),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f6".to_string()],
                    metadata: Some(Metadata {
                        title: Some(Arc::new("f6 title".to_string())),
                        summary: None,
                        authors: Default::default(),
                        tags: HashSet::from_iter(IntoIter::new([Arc::new("t6".to_string())])),
                        publishing_date: Some(500),
                        last_edit_date: None,
                        data: HashMap::default(),
                    }),
                    content: "".to_string(),
                }),
            ],
        });
        let test_folder = TmpTestFolder::new().unwrap();
        test_folder
            .write(&FileNode::Dir {
                name: "templates".to_string(),
                sub: vec![FileNode::File {
                    name: "asset.index.html.hbs".to_string(),
                    content: indoc! {"
                            {{#each (bundle_query \"{tag: t1}\" \"{limit: 2}\") }}
                            <h1>{{this.metadata.title}}</h1>
                            {{/each}}
                            {{#each (bundle_query \"{tag: t1}\" \"{limit: 10}\") }}
                            <h2>{{this.metadata.title}}</h2>
                            {{/each}}
                            {{#each (bundle_query \"{tag: t1}\" \"{skip: 2, limit: 1}\") }}
                            <h3>{{this.metadata.title}}</h3>
                            {{/each}}
                            {{#each (bundle_query \"\" \"{skip: 4}\") }}
                            <h4>{{this.metadata.title}}</h4>
                            {{/each}}"}
                    .as_bytes()
                    .to_vec(),
                    open_options: None,
                }],
            })
            .unwrap();

        let stages = AppendStage {
            name: "append".to_string(),
            inner: Arc::new(HandlebarsStage {
                name: "hb stage".to_string(),
                lookup: Arc::new(HandlebarsDir {
                    base_path: test_folder.get_path().join("templates"),
                }),
            }),
        };

        let result_bundle = stages.process(&bundle, &Env::test()).unwrap();

        let index_page = result_bundle
            .0
            .pages()
            .iter()
            .find(|p| p.path() == vec!["index.html"].as_slice())
            .map(|p| {
                let mut content: String = "".to_string();
                p.open(&PageIndex::from(p), &BundleIndex::from(&result_bundle.0), &Env::test())
                    .unwrap()
                    .read_to_string(&mut content)
                    .unwrap();
                content
            })
            .unwrap();
        assert_eq!(
            index_page,
            indoc! {"
                <h1>f5 title</h1>
                <h1>f4 title</h1>
                <h2>f5 title</h2>
                <h2>f4 title</h2>
                <h2>f3 title</h2>
                <h3>f3 title</h3>
                <h4>f4 title</h4>
                <h4>f3 title</h4>
                "
            }
        );
    }

    #[derive(Debug)]
    struct NewHandlebarsLookupTest {
        registry: handlebars::Handlebars<'static>,
        fetch: Option<String>,
        assets: Vec<Arc<dyn Page>>,
        template_assets: Vec<TemplateAsset>,
    }

    #[derive(Debug)]
    struct NewHandlebarsLookupResultTest {
        registry: handlebars::Handlebars<'static>,
        fetch: Option<String>,
        assets: Vec<Arc<dyn Page>>,
        template_assets: Vec<TemplateAsset>,
    }

    impl HandlebarsLookupResult for NewHandlebarsLookupResultTest {
        fn clone_registry(&self) -> handlebars::Handlebars<'static> {
            self.registry.clone()
        }

        fn fetch(&self, _: &Arc<dyn Page>) -> Option<String> {
            self.fetch.clone()
        }

        fn assets(&self) -> Vec<Arc<dyn Page>> {
            self.assets.clone()
        }

        fn template_assets(&self) -> Vec<TemplateAsset> {
            self.template_assets.clone()
        }
    }

    impl HandlebarsLookup for NewHandlebarsLookupTest {
        fn lookup(&self, _: &Env) -> anyhow::Result<Arc<dyn HandlebarsLookupResult>> {
            Ok(Arc::new(NewHandlebarsLookupResultTest {
                registry: self.registry.clone(),
                fetch: self.fetch.clone(),
                assets: self.assets.clone(),
                template_assets: self.template_assets.clone(),
            }))
        }
    }
}

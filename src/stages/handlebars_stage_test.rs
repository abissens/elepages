#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, Metadata, Page, PageBundle, VecBundle};
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{HandlebarsDir, HandlebarsLookup, HandlebarsLookupResult, HandlebarsStage, Stage, TemplateAsset};
    use rustassert::fs::{FileNode, TmpTestFolder};
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
        registry.register_template_string("tpl_1", "TPL 1 : {{title}} \n {{content_as_string}}").unwrap();
        let hb_stage = HandlebarsStage {
            name: "hb stage".to_string(),
            lookup: Arc::new(NewHandlebarsLookupTest {
                registry,
                fetch: Some("tpl_1".to_string()),
                assets: vec![],
                template_assets: vec![],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
        registry.register_template_string("tpl_1", "TPL 1 : {{title}} \n {{content_as_string}}").unwrap();
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
                        }),
                        content: "b content".to_string(),
                    }),
                ],
                template_assets: vec![],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
        registry.register_template_string("tpl_1", "TPL 1 : {{title}} \n {{content_as_string}}").unwrap();
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
                        }),
                    },
                ],
            }),
        };

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                    content: "TPL root : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
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

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                        content: "TPL root : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![FileNode::File {
                            name: "page.hbs".to_string(),
                            content: "TPL dir : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
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

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                        content: "TPL root : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "TPL f4 : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
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

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                        content: "TPL root : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "base.hbs".to_string(),
                        content: "TPL base : {{title}} \n {{> page}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::Dir {
                        name: "dir".to_string(),
                        sub: vec![
                            FileNode::File {
                                name: "base.hbs".to_string(),
                                content: "TPL base 2 : {{title}} \n {{> page}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "{{#> base}}{{#*inline \"page\"}}inner: {{content_as_string}}{{/inline}} {{/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f5.html.hbs".to_string(),
                                content: "{{#> dir/base}}{{#*inline \"page\"}}inner: {{content_as_string}}{{/inline}} {{/dir/base}}".as_bytes().to_vec(),
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

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                        content: "TPL root : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                        open_options: None,
                    },
                    FileNode::File {
                        name: "base.hbs".to_string(),
                        content: "TPL base : {{title}} \n {{> page}}".as_bytes().to_vec(),
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
                                content: "TPL base 2 : {{title}} \n {{> page}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.hbs".to_string(),
                                content: "TPL dir : {{title}} \n {{content_as_string}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f4.html.hbs".to_string(),
                                content: "{{#> base}}{{#*inline \"page\"}}inner: {{content_as_string}}{{/inline}} {{/base}}".as_bytes().to_vec(),
                                open_options: None,
                            },
                            FileNode::File {
                                name: "page.f5.html.hbs".to_string(),
                                content: "{{#> dir/base}}{{#*inline \"page\"}}inner: {{content_as_string}}{{/inline}} {{/dir/base}}".as_bytes().to_vec(),
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

        let result_bundle = hb_stage.process(&bundle, &Env::new()).unwrap();
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
                    metadata: None,
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
                    metadata: None,
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
                    metadata: None,
                    content: "test index".to_string(),
                },
            ]
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
        fn lookup(&self) -> anyhow::Result<Arc<dyn HandlebarsLookupResult>> {
            Ok(Arc::new(NewHandlebarsLookupResultTest {
                registry: self.registry.clone(),
                fetch: self.fetch.clone(),
                assets: self.assets.clone(),
                template_assets: self.template_assets.clone(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, ExtSelector, PageBundle, PathSelector, VecBundle};
    use crate::stages::compose_stage::ComposeStage;
    use crate::stages::compose_stage::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::copy_cut_stage::CopyCut;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use std::sync::Arc;

    #[test]
    fn compose_stage_should_create_new_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                name: "copy stage".to_string(),
                dest: vec!["copied".to_string()],
                selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
            })))],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();

        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "copy stage".to_string(),
                    sub_results: Default::default()
                }]
            }
        );

        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compose_stage_should_replace_sub_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![Arc::new(ReplaceSubSet(
                Arc::new(PathSelector {
                    query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                }),
                Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }),
            ))],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "copy stage".to_string(),
                    sub_results: Default::default()
                }]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compose_stage_should_create_and_replace_sub_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![
                Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["backup".to_string(), "copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }))),
                Arc::new(ReplaceSubSet(
                    Arc::new(PathSelector {
                        query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                    }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
            ],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    }
                ]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn compose_stage_should_create_and_replace_sub_page_set_based_on_multiple_selectors() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![
                Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["backup".to_string(), "copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }))),
                Arc::new(ReplaceSubSet(
                    Arc::new(PathSelector {
                        query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                    }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Arc::new(ExtSelector { ext: ".md".to_string() }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied ext".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
            ],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    }
                ]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied ext".to_string(), "a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parallel_compose_stage_should_create_new_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                name: "copy stage".to_string(),
                dest: vec!["copied".to_string()],
                selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
            })))],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "copy stage".to_string(),
                    sub_results: Default::default()
                },]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parallel_compose_stage_should_replace_sub_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![Arc::new(ReplaceSubSet(
                Arc::new(PathSelector {
                    query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                }),
                Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }),
            ))],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![TestProcessingResult {
                    stage_name: "copy stage".to_string(),
                    sub_results: Default::default()
                },]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parallel_compose_stage_should_create_and_replace_sub_page_set() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![
                Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["backup".to_string(), "copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }))),
                Arc::new(ReplaceSubSet(
                    Arc::new(PathSelector {
                        query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                    }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
            ],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                ]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parallel_compose_stage_should_create_and_replace_sub_page_set_based_on_multiple_selectors() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let compose_stage = ComposeStage {
            name: "compose stage".to_string(),
            units: vec![
                Arc::new(CreateNewSet(Arc::new(CopyCut::Move {
                    name: "copy stage".to_string(),
                    dest: vec!["backup".to_string(), "copied".to_string()],
                    selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                }))),
                Arc::new(ReplaceSubSet(
                    Arc::new(PathSelector {
                        query: vec!["d1".to_string(), "d2".to_string(), "**".to_string()],
                    }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Arc::new(ExtSelector { ext: ".md".to_string() }),
                    Arc::new(CopyCut::Move {
                        name: "copy stage".to_string(),
                        dest: vec!["copied ext".to_string()],
                        selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
                    }),
                )),
            ],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "compose stage".to_string(),
                sub_results: vec![
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    },
                    TestProcessingResult {
                        stage_name: "copy stage".to_string(),
                        sub_results: Default::default()
                    }
                ]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["backup".to_string(), "copied".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied ext".to_string(), "a.md".to_string()],
                    metadata: None,
                    content: "test content a md".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }
}

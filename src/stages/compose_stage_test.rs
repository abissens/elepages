#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{PageBundle, VecBundle};
    use crate::stages::compose_stage::ComposeUnit::{CreateNewSet, ReplaceSubSet};
    use crate::stages::compose_stage::{ComposeStage, ExtSelector, PrefixSelector, RegexSelector};
    use crate::stages::copy_stage::CopyStage;
    use crate::stages::stage::Stage;
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
            units: vec![Arc::new(CreateNewSet(Arc::new(CopyStage {
                name: "copy stage".to_string(),
                prefix: vec!["copied".to_string()],
            })))],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["copied".to_string()],
                }),
            ))],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Arc::new(CreateNewSet(Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["backup".to_string(), "copied".to_string()],
                }))),
                Arc::new(ReplaceSubSet(
                    Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied".to_string()],
                    }),
                )),
            ],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Arc::new(CreateNewSet(Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["backup".to_string(), "copied".to_string()],
                }))),
                Arc::new(ReplaceSubSet(
                    Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied".to_string()],
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Box::new(RegexSelector(regex::Regex::new(r"^.*?f\d$").unwrap())),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied regex".to_string()],
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Box::new(ExtSelector(".md".into())),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied ext".to_string()],
                    }),
                )),
            ],
            parallel: false,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                    path: vec!["copied regex".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "f2".to_string()],
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
            units: vec![Arc::new(CreateNewSet(Arc::new(CopyStage {
                name: "copy stage".to_string(),
                prefix: vec!["copied".to_string()],
            })))],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["copied".to_string()],
                }),
            ))],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Arc::new(CreateNewSet(Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["backup".to_string(), "copied".to_string()],
                }))),
                Arc::new(ReplaceSubSet(
                    Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied".to_string()],
                    }),
                )),
            ],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                Arc::new(CreateNewSet(Arc::new(CopyStage {
                    name: "copy stage".to_string(),
                    prefix: vec!["backup".to_string(), "copied".to_string()],
                }))),
                Arc::new(ReplaceSubSet(
                    Box::new(PrefixSelector(vec!["d1".to_string(), "d2".to_string()])),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied".to_string()],
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Box::new(RegexSelector(regex::Regex::new(r"^.*?f\d$").unwrap())),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied regex".to_string()],
                    }),
                )),
                Arc::new(ReplaceSubSet(
                    Box::new(ExtSelector(".md".into())),
                    Arc::new(CopyStage {
                        name: "copy stage".to_string(),
                        prefix: vec!["copied ext".to_string()],
                    }),
                )),
            ],
            parallel: true,
        };

        let result_bundle = compose_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                    path: vec!["copied regex".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "d2".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["copied regex".to_string(), "d1".to_string(), "f2".to_string()],
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
            ]
        );
    }
}

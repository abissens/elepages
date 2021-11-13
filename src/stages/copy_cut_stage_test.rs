#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{PageBundle, PathSelector, VecBundle};
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::CopyCut;
    use std::sync::Arc;

    #[test]
    fn copy_selected_pages() {
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
                    path: vec!["d3".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let stage = CopyCut::Copy {
            name: "copy cut stage".to_string(),
            selector: Arc::new(PathSelector {
                query: vec!["d1".to_string(), "**".to_string()],
            }),
            dest: vec!["copied".to_string()],
        };

        let result_bundle = stage.process(&bundle).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "copy cut stage".to_string(),
                sub_results: vec![]
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
                    path: vec!["d1".to_string(), "d2".to_string(), "f3".to_string()],
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
                TestPage {
                    path: vec!["d3".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn move_selected_pages() {
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
                    path: vec!["d3".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let stage = CopyCut::Move {
            name: "copy cut stage".to_string(),
            selector: Arc::new(PathSelector {
                query: vec!["d1".to_string(), "**".to_string()],
            }),
            dest: vec!["moved".to_string()],
        };

        let result_bundle = stage.process(&bundle).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "copy cut stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["d3".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["moved".to_string(), "d1".to_string(), "d2".to_string(), "f3".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["moved".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["moved".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn ignore_selected_pages() {
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
                    path: vec!["d3".to_string(), "f4".to_string()],
                    metadata: None,
                    content: "".to_string(),
                }),
            ],
        });

        let stage = CopyCut::Ignore {
            name: "copy cut stage".to_string(),
            selector: Arc::new(PathSelector {
                query: vec!["d1".to_string(), "**".to_string()],
            }),
        };

        let result_bundle = stage.process(&bundle).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "copy cut stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[TestPage {
                path: vec!["d3".to_string(), "f4".to_string()],
                metadata: None,
                content: "".to_string(),
            },]
        );
    }
}

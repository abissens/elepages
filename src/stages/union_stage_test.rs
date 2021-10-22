#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{PageBundle, VecBundle};
    use crate::stages::copy_stage::CopyStage;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestStage;
    use crate::stages::union_stage::UnionStage;
    use std::borrow::Borrow;
    use std::sync::Arc;

    #[test]
    fn parallel_union_stage_should_merge_two_copy_sub_stages() {
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

        let copy_stage_1 = CopyStage {
            prefix: vec!["root".to_string(), "sub_root".to_string()],
        };

        let copy_stage_2 = CopyStage {
            prefix: vec!["second_root".to_string()],
        };

        let union_stage = UnionStage {
            stages: vec![Arc::new(copy_stage_1), Arc::new(copy_stage_2)],
            parallel: true,
        };

        let result_bundle = union_stage.process(bundle.borrow()).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn sequential_union_stage_should_merge_two_copy_sub_stages() {
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

        let copy_stage_1 = CopyStage {
            prefix: vec!["root".to_string(), "sub_root".to_string()],
        };

        let copy_stage_2 = CopyStage {
            prefix: vec!["second_root".to_string()],
        };

        let union_stage = UnionStage {
            stages: vec![Arc::new(copy_stage_1), Arc::new(copy_stage_2)],
            parallel: false,
        };

        let result_bundle = union_stage.process(bundle.borrow()).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["root".to_string(), "sub_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f1".to_string()],
                    metadata: None,
                    content: "test content".to_string(),
                },
                TestPage {
                    path: vec!["second_root".to_string(), "d1".to_string(), "f2".to_string()],
                    metadata: None,
                    content: "".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parallel_union_stage_should_return_error_on_errored_stage_and_cancel_non_launched_stage() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle { p: vec![] });

        let ok_stage: Arc<dyn Stage> = Arc::new(TestStage::ok(Arc::new(VecBundle {
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
        })));

        let err_stage = TestStage::err("some error");

        let union_stage = UnionStage {
            stages: vec![
                Arc::clone(&ok_stage),
                Arc::new(err_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
                Arc::clone(&ok_stage),
            ],
            parallel: true,
        };

        let result_bundle = union_stage.process(bundle.borrow());
        assert!(matches!(result_bundle, Err(e) if e.to_string() == "some error"));
        if let Some(r) = ok_stage.as_any().downcast_ref::<TestStage>() {
            println!("{}", *r.launched.lock().unwrap());
        } else {
            panic!("TestStage should be downcasted");
        }
    }
}

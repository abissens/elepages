#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{PageBundle, VecBundle};
    use crate::stages::copy_stage::CopyStage;
    use crate::stages::stage::Stage;
    use std::sync::Arc;

    #[test]
    fn copy_stage_should_copy_all_bundle_paths_to_another_root_path() {
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
        let copy_stage = CopyStage {
            prefix: vec!["root".to_string(), "sub_root".to_string()],
        };

        let result_bundle = copy_stage.process(&bundle).unwrap();

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
            ]
        );
    }
}

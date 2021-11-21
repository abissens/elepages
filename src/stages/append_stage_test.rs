#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, PageBundle, PathSelector, VecBundle};
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::{AppendStage, CopyCut, Stage};
    use std::sync::Arc;

    #[test]
    fn append_inner_stage_bundle_to_current_bundle() {
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

        let compose_stage = AppendStage {
            name: "append stage".to_string(),
            inner: Arc::new(CopyCut::Move {
                name: "copy stage".to_string(),
                dest: vec!["copied".to_string()],
                selector: Arc::new(PathSelector { query: vec!["**".to_string()] }),
            }),
        };

        let result_bundle = compose_stage.process(&bundle, &Env::test()).unwrap();

        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "append stage".to_string(),
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
}

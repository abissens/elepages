#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Env, PageBundle, VecBundle};
    use crate::stages::md_stage::MdStage;
    use crate::stages::stage::Stage;
    use crate::stages::test_stage::TestProcessingResult;
    use crate::stages::PageGeneratorBagImpl;
    use indoc::indoc;
    use std::sync::Arc;

    #[test]
    fn transform_md_pages_to_html_ones() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.md".to_string()],
                    metadata: None,
                    content: indoc! {"
                        paragraph 1
                        paragraph 1

                        paragraph 2
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["f2.md".to_string()],
                    metadata: None,
                    content: indoc! {"
                        An H1 Header
                        ============

                        An H2 Header
                        ------------
                    "}
                    .to_string(),
                }),
                Arc::new(TestPage {
                    path: vec!["dir".to_string(), "f3".to_string()],
                    metadata: None,
                    content: indoc! {"
                        Indented code

                            // Some comments
                            line 1 of code
                            line 2 of code
                            line 3 of code
                    "}
                    .to_string(),
                }),
            ],
        });

        let md_stage = MdStage { name: "md stage".to_string() };
        let result_bundle = md_stage.process(&bundle, &Env::test(), &PageGeneratorBagImpl::new()).unwrap();
        assert_eq!(
            TestProcessingResult::from(&result_bundle.1),
            TestProcessingResult {
                stage_name: "md stage".to_string(),
                sub_results: vec![]
            }
        );
        let mut actual = result_bundle.0.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
        actual.sort_by_key(|f| f.path.join("/"));
        assert_eq!(
            actual,
            &[
                TestPage {
                    path: vec!["dir".to_string(), "f3".to_string()],
                    metadata: None,
                    content: indoc! {"
                        <p>Indented code</p>
                        <pre><code>// Some comments
                        line 1 of code
                        line 2 of code
                        line 3 of code
                        </code></pre>
                        "}
                    .to_string()
                },
                TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: None,
                    content: indoc! {"
                        <p>paragraph 1
                        paragraph 1</p>
                        <p>paragraph 2</p>
                    "}
                    .to_string()
                },
                TestPage {
                    path: vec!["f2.html".to_string()],
                    metadata: None,
                    content: indoc! {"
                        <h1>An H1 Header</h1>
                        <h2>An H2 Header</h2>
                    "}
                    .to_string()
                },
            ]
        );
    }
}

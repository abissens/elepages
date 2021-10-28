#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Metadata, Page, PageBundle, VecBundle};
    use crate::stages::handlebars_stage::{HandlebarsLookup, HandlebarsStage};
    use crate::stages::stage::Stage;
    use std::sync::Arc;

    #[derive(Debug)]
    struct LookupTest();

    impl HandlebarsLookup for LookupTest {
        fn init_registry(&self, registry: &mut handlebars::Handlebars) {
            registry.register_template_string("tpl_1", "TPL 1 : {{title}} \n {{content_as_string}}").unwrap();
        }

        fn fetch(&self, _: &Arc<dyn Page>) -> Option<String> {
            Some("tpl_1".to_string())
        }
    }
    #[test]
    fn apply_handle_bar_template_to_bundle() {
        let bundle: Arc<dyn PageBundle> = Arc::new(VecBundle {
            p: vec![
                Arc::new(TestPage {
                    path: vec!["f1.html".to_string()],
                    metadata: Some(Metadata {
                        title: Some("f1 title".to_string()),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
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

        let hb_stage = HandlebarsStage { lookup: Arc::new(LookupTest()) };

        let result_bundle = hb_stage.process(&bundle).unwrap();

        let mut actual = result_bundle.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
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
                        title: Some("f1 title".to_string()),
                        summary: None,
                        authors: Default::default(),
                        tags: Default::default(),
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
}

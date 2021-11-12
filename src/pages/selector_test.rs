#[cfg(test)]
mod tests {
    use crate::pages::selector::{PathSelector, Selector};
    use crate::pages::test_page::TestPage;
    use crate::pages::{ExtSelector, Metadata, PageBundle, TagSelector, VecBundle};
    use std::sync::Arc;

    #[macro_export]
    macro_rules! path_bundle {
        ($($result:expr),+) => {
            Arc::new(VecBundle {
                p: vec![
                    $(Arc::new(TestPage {
                        path: $result.iter().map(|s| s.to_string()).collect(),
                        metadata: None,
                        content: "".to_string(),
                    })), +
                ],
            })
        };
    }

    #[macro_export]
    macro_rules! tag_bundle {
        ($($result:expr),+) => {
            Arc::new(VecBundle {
                p: vec![
                    $(Arc::new(TestPage {
                        path: vec![],
                        metadata: Some(Metadata{
                            title: None,
                            summary: None,
                            authors: Default::default(),
                            tags: $result.iter().map(|s| Arc::new(s.to_string())).collect(),
                            publishing_date: None,
                            last_edit_date: None
                        }),
                        content: "".to_string()
                })), +
                ],
            })
        };
    }

    #[macro_export]
    macro_rules! assert_eq_bundles {
        ($bundle_1:expr, $bundle_2:expr) => {
            let mut b1 = $bundle_1.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
            b1.sort_by_key(|f| f.path.join("/"));

            let mut b2 = $bundle_2.pages().iter().map(|p| TestPage::from(p)).collect::<Vec<_>>();
            b2.sort_by_key(|f| f.path.join("/"));
            assert_eq!(b1, b2)
        };
    }

    #[test]
    fn select_all_pages_when_query_is_empty() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"]);
        let selector = PathSelector { query: vec![] };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"]));
    }

    #[test]
    fn select_page_by_its_path() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d1", "f3"]);
        let selector = PathSelector {
            query: vec!["d1".to_string(), "f2".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f2"]));
    }

    #[test]
    fn select_pages_by_their_name_pattern() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f2"], vec!["d1", "f3.txt"]);
        let selector = PathSelector {
            query: vec!["d1".to_string(), "*.txt".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f3.txt"]));
    }

    #[test]
    fn multi_name_pattern() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f2"], vec!["d1", "f3.txt"], vec!["f4.txt"], vec!["d.txt"]);
        let selector = PathSelector {
            query: vec!["**".to_string(), "f*.t*t".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f3.txt"], vec!["f4.txt"]));
    }

    #[test]
    fn adjacent_stars_on_name_pattern() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f2"], vec!["d1", "f3.txt"], vec!["f4.txt"], vec!["d.txt"]);
        let selector = PathSelector {
            query: vec!["**".to_string(), "f***.t**t".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1.txt"], vec!["d1", "f3.txt"], vec!["f4.txt"]));
    }

    #[test]
    fn select_multiple_same_level_pages() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d2", "f3"]);
        let selector = PathSelector {
            query: vec!["d1".to_string(), "*".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"]));
    }

    #[test]
    fn select_multiple_different_level_pages() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d1", "d2", "f2"], vec!["d3", "f3"]);
        let selector = PathSelector {
            query: vec!["d1".to_string(), "**".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d1", "d2", "f2"]));
    }

    #[test]
    fn select_pages_based_on_pattern() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d1", "d2", "f2"], vec!["d1", "d2", "d3", "f1"], vec!["d3", "f3"]);
        let selector = PathSelector {
            query: vec!["d1".to_string(), "**".to_string(), "f1".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1"], vec!["d1", "d2", "d3", "f1"]));
    }

    #[test]
    fn select_pages_based_on_multiple_patterns() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(
            vec!["d1", "f1"],
            vec!["d1", "f2"],
            vec!["d1", "d2", "f2"],
            vec!["d1", "d2", "d3", "f1"],
            vec!["d3", "f3"],
            vec!["d3", "d1", "f4"]
        );

        let selector = PathSelector {
            query: vec!["**".to_string(), "d1".to_string(), "**".to_string()],
        };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(
            result_bundle,
            path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"], vec!["d1", "d2", "f2"], vec!["d1", "d2", "d3", "f1"], vec!["d3", "d1", "f4"])
        );
    }

    #[test]
    fn select_all_pages_when_ext_is_empty() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"]);
        let selector = ExtSelector { ext: "".to_string() };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f1"], vec!["d1", "f2"]));
    }

    #[test]
    fn select_pages_by_their_ext() {
        let bundle: Arc<dyn PageBundle> = path_bundle!(vec!["d1", "f1"], vec!["d1", "f2.md"], vec!["d1", "f3"], vec!["f4.md".to_string()]);
        let selector = ExtSelector { ext: ".md".to_string() };

        let result_bundle = selector.select(&bundle);

        assert_eq_bundles!(result_bundle, path_bundle!(vec!["d1", "f2.md"], vec!["f4.md".to_string()]));
    }

    #[test]
    fn select_pages_by_tags() {
        let bundle: Arc<dyn PageBundle> = tag_bundle!(vec!["a", "b", "c"], vec!["a"], vec!["e", "f"]);

        let selector_1 = TagSelector { tag: "a".to_string() };
        let selector_2 = TagSelector { tag: "f".to_string() };
        let selector_3 = TagSelector { tag: "g".to_string() };

        let result_bundle_1 = selector_1.select(&bundle);
        let result_bundle_2 = selector_2.select(&bundle);
        let result_bundle_3 = selector_3.select(&bundle);

        assert_eq_bundles!(result_bundle_1, tag_bundle!(vec!["a", "b", "c"], vec!["a"]));
        assert_eq_bundles!(result_bundle_2, tag_bundle!(vec!["e", "f"]));
        assert!(result_bundle_3.pages().is_empty());
    }
}

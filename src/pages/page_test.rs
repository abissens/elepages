#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Author, Metadata, Page, PageProxy};
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn page_proxy_preserves_path_name() {
        let test_page: Arc<dyn Page> = Arc::new(TestPage {
            path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            metadata: None,
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: None,
            new_metadata: None,
            inner: Arc::clone(&test_page),
        };

        assert_eq!(proxy.path(), &vec!["a".to_string(), "b".to_string(), "c".to_string()])
    }

    #[test]
    fn page_proxy_changes_path_name() {
        let test_page: Arc<dyn Page> = Arc::new(TestPage {
            path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            metadata: None,
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: Some(vec!["e".to_string(), "f".to_string(), "g".to_string()]),
            new_metadata: None,
            inner: Arc::clone(&test_page),
        };

        assert_eq!(proxy.path(), &vec!["e".to_string(), "f".to_string(), "g".to_string()])
    }

    #[test]
    fn page_proxy_preserves_metadata() {
        let test_page: Arc<dyn Page> = Arc::new(TestPage {
            path: vec![],
            metadata: Some(Metadata {
                title: Some("test page".to_string()),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
            }),
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: None,
            new_metadata: None,
            inner: Arc::clone(&test_page),
        };

        assert!(matches!(proxy.metadata(), Some(metadata) if metadata == &Metadata{
            title: Some("test page".to_string()),
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
        }))
    }

    #[test]
    fn page_proxy_changes_metadata() {
        let test_page: Arc<dyn Page> = Arc::new(TestPage {
            path: vec![],
            metadata: Some(Metadata {
                title: Some("test page".to_string()),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
            }),
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: None,
            new_metadata: Some(Metadata {
                title: Some("new test page".to_string()),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
            }),
            inner: Arc::clone(&test_page),
        };

        assert!(matches!(proxy.metadata(), Some(metadata) if metadata == &Metadata{
            title: Some("new test page".to_string()),
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
        }))
    }

    #[test]
    fn metadata_merge_title_and_summary_attributes() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
        };

        let m2 = Metadata {
            title: Some("title".to_string()),
            summary: Some("summary".to_string()),
            authors: HashSet::new(),
            tags: HashSet::new(),
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(result, m2);
    }

    #[test]
    fn metadata_merge_should_use_self_as_base_metadata() {
        let m1 = Metadata {
            title: Some("title".to_string()),
            summary: Some("summary".to_string()),
            authors: vec![Author {
                name: "a1".to_string(),
                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
            }]
            .iter()
            .map(|x| x.clone())
            .collect(),
            tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
        };

        let m2 = Metadata {
            title: Some("parent title".to_string()),
            summary: Some("parent summary".to_string()),
            authors: HashSet::new(),
            tags: HashSet::new(),
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(result, m1);
    }

    #[test]
    fn metadata_merge_author_contacts_when_common_authors() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: vec![
                Author {
                    name: "a1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a2".to_string(),
                    contacts: HashSet::new(),
                },
                Author {
                    name: "a3".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a4".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                },
            ]
            .iter()
            .map(|x| x.clone())
            .collect(),
            tags: HashSet::new(),
        };

        let m2 = Metadata {
            title: None,
            summary: None,
            authors: vec![
                Author {
                    name: "a1".to_string(),
                    contacts: vec!["c3"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a2".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a3".to_string(),
                    contacts: HashSet::new(),
                },
                Author {
                    name: "b1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                },
            ]
            .iter()
            .map(|x| x.clone())
            .collect(),
            tags: HashSet::new(),
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(
            result,
            Metadata {
                title: None,
                summary: None,
                authors: vec![
                    Author {
                        name: "a1".to_string(),
                        contacts: vec!["c1", "c2", "c3"].iter().map(|x| x.to_string()).collect()
                    },
                    Author {
                        name: "a2".to_string(),
                        contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect()
                    },
                    Author {
                        name: "a3".to_string(),
                        contacts: vec!["c1"].iter().map(|x| x.to_string()).collect()
                    },
                    Author {
                        name: "a4".to_string(),
                        contacts: vec!["c1"].iter().map(|x| x.to_string()).collect()
                    },
                ]
                .iter()
                .map(|x| x.clone())
                .collect(),
                tags: HashSet::new(),
            }
        );
    }

    #[test]
    fn metadata_merge_tags() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
        };

        let m2 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: vec!["t3", "t4"].iter().map(|x| x.to_string()).collect(),
        };

        let m3 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: vec!["t2", "t3"].iter().map(|x| x.to_string()).collect(),
        };

        let m4 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
        };

        let result1 = m1.merge(&m2).unwrap();
        let result2 = m1.merge(&m3).unwrap();
        let result3 = m1.merge(&m4).unwrap();
        let result4 = m4.merge(&m1).unwrap();

        assert_eq!(
            result1,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: vec!["t1", "t2", "t3", "t4"].iter().map(|x| x.to_string()).collect()
            }
        );

        assert_eq!(
            result2,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: vec!["t1", "t2", "t3"].iter().map(|x| x.to_string()).collect()
            }
        );

        assert_eq!(
            result3,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect()
            }
        );

        assert_eq!(
            result4,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect()
            }
        );
    }

    #[test]
    fn metadata_merging_self_returns_a_clone() {
        let m = Metadata {
            title: Some("title".to_string()),
            summary: Some("summary".to_string()),
            authors: vec![
                Author {
                    name: "a1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a2".to_string(),
                    contacts: HashSet::new(),
                },
                Author {
                    name: "a3".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                },
                Author {
                    name: "a4".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                },
            ]
            .iter()
            .map(|x| x.clone())
            .collect(),
            tags: vec!["t1", "t2"].iter().map(|x| x.to_string()).collect(),
        };

        let result = m.merge(&m).unwrap();

        assert_eq!(result, m);
    }
}

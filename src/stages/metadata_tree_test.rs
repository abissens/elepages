#[cfg(test)]
mod tests {
    use crate::pages::Metadata;
    use crate::stages::metadata_tree::{MetadataNode, MetadataTree};
    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use std::sync::Arc;

    fn new_metadata(title: &str) -> Metadata {
        Metadata {
            title: Some(Arc::new(title.to_string())),
            summary: None,
            authors: Default::default(),
            tags: Default::default(),
            publishing_date: None,
            last_edit_date: None,
            data: HashMap::default(),
        }
    }

    #[test]
    fn metadata_tree_prevent_empty_path_push() {
        let mut tree = MetadataTree::Root { sub: Default::default() };

        let r = tree.push(&vec![], new_metadata("title"));

        assert!(matches!(r, Err(b) if b.to_string() == "path cannot be empty on root node"));
    }

    #[test]
    fn metadata_tree_push_single_metadata_path() {
        let mut tree = MetadataTree::Root { sub: Default::default() };

        tree.push(&vec!["a.txt".to_string()], new_metadata("title")).unwrap();

        assert_eq!(
            tree,
            MetadataTree::Root {
                sub: HashMap::from_iter(IntoIter::new([(
                    "a.txt".to_string(),
                    MetadataTree::Node {
                        metadata: Some(new_metadata("title")),
                        sub: Default::default()
                    }
                )]))
            }
        )
    }

    #[test]
    fn metadata_tree_push_multiple_first_level_metadata_path() {
        let mut tree = MetadataTree::Root { sub: Default::default() };

        tree.push(&vec!["a.txt".to_string()], new_metadata("title 1")).unwrap();
        tree.push(&vec!["b.txt".to_string()], new_metadata("title 2")).unwrap();
        tree.push(&vec!["c".to_string()], new_metadata("title 3")).unwrap();

        assert_eq!(
            tree,
            MetadataTree::Root {
                sub: HashMap::from_iter(IntoIter::new([
                    (
                        "a.txt".to_string(),
                        MetadataTree::Node {
                            metadata: Some(new_metadata("title 1")),
                            sub: Default::default()
                        }
                    ),
                    (
                        "b.txt".to_string(),
                        MetadataTree::Node {
                            metadata: Some(new_metadata("title 2")),
                            sub: Default::default()
                        }
                    ),
                    (
                        "c".to_string(),
                        MetadataTree::Node {
                            metadata: Some(new_metadata("title 3")),
                            sub: Default::default()
                        }
                    ),
                ]))
            }
        )
    }

    #[test]
    fn metadata_tree_push_multiple_multi_level_metadata_path() {
        let mut tree = MetadataTree::Root { sub: Default::default() };

        tree.push(&vec!["a.txt".to_string()], new_metadata("title 1")).unwrap();
        tree.push(&vec!["a".to_string(), "b.txt".to_string()], new_metadata("title 2")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string()], new_metadata("title 3")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "c".to_string()], new_metadata("title 4")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "d".to_string()], new_metadata("title 5")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "e".to_string(), "f".to_string()], new_metadata("title 6")).unwrap();

        assert_eq!(
            tree,
            MetadataTree::Root {
                sub: HashMap::from_iter(IntoIter::new([
                    (
                        "a.txt".to_string(),
                        MetadataTree::Node {
                            metadata: Some(new_metadata("title 1")),
                            sub: Default::default()
                        }
                    ),
                    (
                        "a".to_string(),
                        MetadataTree::Node {
                            metadata: None,
                            sub: HashMap::from_iter(IntoIter::new([
                                (
                                    "b.txt".to_string(),
                                    MetadataTree::Node {
                                        metadata: Some(new_metadata("title 2")),
                                        sub: Default::default()
                                    }
                                ),
                                (
                                    "b".to_string(),
                                    MetadataTree::Node {
                                        metadata: Some(new_metadata("title 3")),
                                        sub: HashMap::from_iter(IntoIter::new([
                                            (
                                                "c".to_string(),
                                                MetadataTree::Node {
                                                    metadata: Some(new_metadata("title 4")),
                                                    sub: Default::default()
                                                }
                                            ),
                                            (
                                                "d".to_string(),
                                                MetadataTree::Node {
                                                    metadata: Some(new_metadata("title 5")),
                                                    sub: Default::default()
                                                }
                                            ),
                                            (
                                                "e".to_string(),
                                                MetadataTree::Node {
                                                    metadata: None,
                                                    sub: HashMap::from_iter(IntoIter::new([(
                                                        "f".to_string(),
                                                        MetadataTree::Node {
                                                            metadata: Some(new_metadata("title 6")),
                                                            sub: Default::default()
                                                        }
                                                    ),]))
                                                }
                                            ),
                                        ]))
                                    }
                                )
                            ]))
                        }
                    )
                ]))
            }
        )
    }

    #[test]
    fn metadata_tree_retrieve_metadata_by_path() {
        let mut tree = MetadataTree::Root { sub: Default::default() };

        tree.push(&vec!["a.txt".to_string()], new_metadata("title 1")).unwrap();
        tree.push(&vec!["a".to_string(), "b.txt".to_string()], new_metadata("title 2")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string()], new_metadata("title 3")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "c".to_string()], new_metadata("title 4")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "d".to_string()], new_metadata("title 5")).unwrap();
        tree.push(&vec!["a".to_string(), "b".to_string(), "e".to_string(), "f".to_string()], new_metadata("title 6")).unwrap();

        let mut result = vec![];
        tree.get_metadata_from_path(&["a.txt".to_string()], &mut result);
        assert_eq!(
            result,
            vec![MetadataNode {
                path: "a.txt".to_string(),
                metadata: Some(&new_metadata("title 1"))
            }]
        );

        let mut result = vec![];
        tree.get_metadata_from_path(&["a".to_string(), "b.txt".to_string()], &mut result);
        assert_eq!(
            result,
            vec![
                MetadataNode {
                    path: "a".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "b.txt".to_string(),
                    metadata: Some(&new_metadata("title 2"))
                }
            ]
        );

        let mut result = vec![];
        tree.get_metadata_from_path(&["a".to_string(), "b".to_string()], &mut result);
        assert_eq!(
            result,
            vec![
                MetadataNode {
                    path: "a".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "b".to_string(),
                    metadata: Some(&new_metadata("title 3"))
                }
            ]
        );

        let mut result = vec![];
        tree.get_metadata_from_path(&["a".to_string(), "b".to_string(), "c".to_string()], &mut result);
        assert_eq!(
            result,
            vec![
                MetadataNode {
                    path: "a".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "b".to_string(),
                    metadata: Some(&new_metadata("title 3"))
                },
                MetadataNode {
                    path: "c".to_string(),
                    metadata: Some(&new_metadata("title 4"))
                }
            ]
        );

        let mut result = vec![];
        tree.get_metadata_from_path(&["a".to_string(), "b".to_string(), "d".to_string()], &mut result);
        assert_eq!(
            result,
            vec![
                MetadataNode {
                    path: "a".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "b".to_string(),
                    metadata: Some(&new_metadata("title 3"))
                },
                MetadataNode {
                    path: "d".to_string(),
                    metadata: Some(&new_metadata("title 5"))
                }
            ]
        );

        let mut result = vec![];
        tree.get_metadata_from_path(&["a".to_string(), "b".to_string(), "e".to_string(), "f".to_string()], &mut result);
        assert_eq!(
            result,
            vec![
                MetadataNode {
                    path: "a".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "b".to_string(),
                    metadata: Some(&new_metadata("title 3"))
                },
                MetadataNode {
                    path: "e".to_string(),
                    metadata: None
                },
                MetadataNode {
                    path: "f".to_string(),
                    metadata: Some(&new_metadata("title 6"))
                }
            ]
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::pages::{Author, Metadata};
    use chrono::DateTime;
    use std::array::IntoIter;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    use std::sync::Arc;

    #[test]
    fn metadata_merge_base_attributes() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: None,
            last_edit_date: None,
        };

        let m2 = Metadata {
            title: Some(Arc::new("title".to_string())),
            summary: Some(Arc::new("summary".to_string())),
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(result, m2);
    }

    #[test]
    fn metadata_merge_should_use_self_as_base_metadata() {
        let m1 = Metadata {
            title: Some(Arc::new("title".to_string())),
            summary: Some(Arc::new("summary".to_string())),
            authors: HashSet::from_iter(IntoIter::new([Arc::new(Author {
                name: "a1".to_string(),
                contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
            })])),
            tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
        };

        let m2 = Metadata {
            title: Some(Arc::new("parent title".to_string())),
            summary: Some(Arc::new("parent summary".to_string())),
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T18:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T19:00:00-08:00").unwrap().timestamp()),
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(result, m1);
    }

    #[test]
    fn metadata_merge_author_contacts_when_common_authors() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::from_iter(IntoIter::new([
                Arc::new(Author {
                    name: "a1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a2".to_string(),
                    contacts: HashSet::new(),
                }),
                Arc::new(Author {
                    name: "a3".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a4".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                }),
            ])),
            tags: HashSet::new(),
            publishing_date: None,
            last_edit_date: None,
        };

        let m2 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::from_iter(IntoIter::new([
                Arc::new(Author {
                    name: "a1".to_string(),
                    contacts: vec!["c3"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a2".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a3".to_string(),
                    contacts: HashSet::new(),
                }),
                Arc::new(Author {
                    name: "b1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                }),
            ])),
            tags: HashSet::new(),
            publishing_date: None,
            last_edit_date: None,
        };

        let result = m1.merge(&m2).unwrap();

        assert_eq!(
            result,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::from_iter(IntoIter::new([
                    Arc::new(Author {
                        name: "a1".to_string(),
                        contacts: vec!["c1", "c2", "c3"].iter().map(|x| x.to_string()).collect()
                    }),
                    Arc::new(Author {
                        name: "a2".to_string(),
                        contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect()
                    }),
                    Arc::new(Author {
                        name: "a3".to_string(),
                        contacts: vec!["c1"].iter().map(|x| x.to_string()).collect()
                    }),
                    Arc::new(Author {
                        name: "a4".to_string(),
                        contacts: vec!["c1"].iter().map(|x| x.to_string()).collect()
                    }),
                ])),
                tags: HashSet::new(),
                publishing_date: None,
                last_edit_date: None,
            }
        );
    }

    #[test]
    fn metadata_merge_tags() {
        let m1 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
            publishing_date: None,
            last_edit_date: None,
        };

        let m2 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::from_iter(IntoIter::new([Arc::new("t3".to_string()), Arc::new("t4".to_string())])),
            publishing_date: None,
            last_edit_date: None,
        };

        let m3 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::from_iter(IntoIter::new([Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
            publishing_date: None,
            last_edit_date: None,
        };

        let m4 = Metadata {
            title: None,
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: None,
            last_edit_date: None,
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
                tags: HashSet::from_iter(IntoIter::new([
                    Arc::new("t1".to_string()),
                    Arc::new("t2".to_string()),
                    Arc::new("t3".to_string()),
                    Arc::new("t4".to_string())
                ])),
                publishing_date: None,
                last_edit_date: None,
            }
        );

        assert_eq!(
            result2,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string()), Arc::new("t3".to_string())])),
                publishing_date: None,
                last_edit_date: None,
            }
        );

        assert_eq!(
            result3,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                publishing_date: None,
                last_edit_date: None,
            }
        );

        assert_eq!(
            result4,
            Metadata {
                title: None,
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
                publishing_date: None,
                last_edit_date: None,
            }
        );
    }

    #[test]
    fn metadata_merging_self_returns_a_clone() {
        let m = Metadata {
            title: Some(Arc::new("title".to_string())),
            summary: Some(Arc::new("summary".to_string())),
            authors: HashSet::from_iter(IntoIter::new([
                Arc::new(Author {
                    name: "a1".to_string(),
                    contacts: vec!["c1", "c2"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a2".to_string(),
                    contacts: HashSet::new(),
                }),
                Arc::new(Author {
                    name: "a3".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                }),
                Arc::new(Author {
                    name: "a4".to_string(),
                    contacts: vec!["c1"].iter().map(|x| x.to_string()).collect(),
                }),
            ])),
            tags: HashSet::from_iter(IntoIter::new([Arc::new("t1".to_string()), Arc::new("t2".to_string())])),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
        };

        let result = m.merge(&m).unwrap();

        assert_eq!(result, m);
    }
}

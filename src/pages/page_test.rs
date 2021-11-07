#[cfg(test)]
mod tests {
    use crate::pages::test_page::TestPage;
    use crate::pages::{Metadata, Page, PageProxy};
    use chrono::DateTime;
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
                title: Some(Arc::new("test page".to_string())),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
                publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
                last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
            }),
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: None,
            new_metadata: None,
            inner: Arc::clone(&test_page),
        };

        assert!(matches!(proxy.metadata(), Some(metadata) if metadata == &Metadata{
            title: Some(Arc::new("test page".to_string())),
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
        }))
    }

    #[test]
    fn page_proxy_changes_metadata() {
        let test_page: Arc<dyn Page> = Arc::new(TestPage {
            path: vec![],
            metadata: Some(Metadata {
                title: Some(Arc::new("test page".to_string())),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
                publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T16:00:00-08:00").unwrap().timestamp()),
                last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T17:00:00-08:00").unwrap().timestamp()),
            }),
            content: "".to_string(),
        });

        let proxy = PageProxy {
            new_path: None,
            new_metadata: Some(Metadata {
                title: Some(Arc::new("new test page".to_string())),
                summary: None,
                authors: HashSet::new(),
                tags: HashSet::new(),
                publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T18:00:00-08:00").unwrap().timestamp()),
                last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T19:00:00-08:00").unwrap().timestamp()),
            }),
            inner: Arc::clone(&test_page),
        };

        assert!(matches!(proxy.metadata(), Some(metadata) if metadata == &Metadata{
            title: Some(Arc::new("new test page".to_string())),
            summary: None,
            authors: HashSet::new(),
            tags: HashSet::new(),
            publishing_date: Some(DateTime::parse_from_rfc3339("2021-10-20T18:00:00-08:00").unwrap().timestamp()),
            last_edit_date: Some(DateTime::parse_from_rfc3339("2021-10-20T19:00:00-08:00").unwrap().timestamp()),
        }))
    }
}

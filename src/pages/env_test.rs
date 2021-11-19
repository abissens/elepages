#[cfg(test)]
mod tests {
    use crate::pages::{Env, PrintLevel, Printer};
    use std::sync::{Arc, Mutex};

    struct TestPrinter {
        prints: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl Printer for TestPrinter {
        fn print(&self, caller: &str, message: &str) {
            self.prints.lock().unwrap().push((caller.to_string(), message.to_string()));
        }
    }

    #[test]
    fn print_level_matches() {
        assert!(PrintLevel::V.should_print(&PrintLevel::V));
        assert!(!PrintLevel::V.should_print(&PrintLevel::VV));
        assert!(!PrintLevel::V.should_print(&PrintLevel::VVV));

        assert!(PrintLevel::VV.should_print(&PrintLevel::V));
        assert!(PrintLevel::VV.should_print(&PrintLevel::VV));
        assert!(!PrintLevel::VV.should_print(&PrintLevel::VVV));

        assert!(PrintLevel::VVV.should_print(&PrintLevel::V));
        assert!(PrintLevel::VVV.should_print(&PrintLevel::VV));
        assert!(PrintLevel::VVV.should_print(&PrintLevel::VVV));
    }

    #[test]
    fn print_nothing_when_print_level_is_none() {
        let messages = Arc::new(Mutex::new(vec![]));
        let env = Env::new(Box::new(TestPrinter { prints: Arc::clone(&messages) }), None);

        env.print(&PrintLevel::V, "caller", "my message 1");
        env.print(&PrintLevel::VV, "caller", "my message 2");
        env.print(&PrintLevel::VVV, "caller", "my message 3");

        assert!(messages.lock().unwrap().is_empty());
    }

    #[test]
    fn print_v_level_messages() {
        let messages = Arc::new(Mutex::new(vec![]));
        let env = Env::new(Box::new(TestPrinter { prints: Arc::clone(&messages) }), Some(PrintLevel::V));

        env.print(&PrintLevel::V, "caller", "my message 1");
        env.print(&PrintLevel::VV, "caller", "my message 2");
        env.print(&PrintLevel::VVV, "caller", "my message 3");

        assert_eq!(messages.lock().unwrap().to_vec(), vec![("caller".to_string(), "my message 1".to_string())]);
    }

    #[test]
    fn print_vv_level_messages() {
        let messages = Arc::new(Mutex::new(vec![]));
        let env = Env::new(Box::new(TestPrinter { prints: Arc::clone(&messages) }), Some(PrintLevel::VV));

        env.print(&PrintLevel::V, "caller", "my message 1");
        env.print(&PrintLevel::VV, "caller", "my message 2");
        env.print(&PrintLevel::VVV, "caller", "my message 3");

        assert_eq!(
            messages.lock().unwrap().to_vec(),
            vec![("caller".to_string(), "my message 1".to_string()), ("caller".to_string(), "my message 2".to_string())]
        );
    }

    #[test]
    fn print_vvv_level_messages() {
        let messages = Arc::new(Mutex::new(vec![]));
        let env = Env::new(Box::new(TestPrinter { prints: Arc::clone(&messages) }), Some(PrintLevel::VVV));

        env.print(&PrintLevel::V, "caller", "my message 1");
        env.print(&PrintLevel::VV, "caller", "my message 2");
        env.print(&PrintLevel::VVV, "caller", "my message 3");

        assert_eq!(
            messages.lock().unwrap().to_vec(),
            vec![
                ("caller".to_string(), "my message 1".to_string()),
                ("caller".to_string(), "my message 2".to_string()),
                ("caller".to_string(), "my message 3".to_string())
            ]
        );
    }
}

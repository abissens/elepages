#[cfg(test)]
mod tests {
    use crate::config::{FromValue, Value};
    use crate::pages_error::PagesError;

    #[test]
    fn string_from_value() {
        let string_v = Value::String("my string".to_string());
        let not_string_v = Value::Bool(true);

        assert_eq!(String::from_value(string_v).unwrap(), "my string".to_string());
        assert_eq!(
            String::from_value(not_string_v).unwrap_err().downcast::<PagesError>().unwrap(),
            PagesError::ValueParsing("expecting Value::String".to_string())
        );
    }

    #[test]
    fn strings_vector_from_value() {
        let string_vec = Value::Vec(vec![Value::String("a".to_string()), Value::String("b".to_string()), Value::String("c".to_string())]);
        let not_vec = Value::Bool(true);
        let not_string_vec = Value::Vec(vec![Value::String("a".to_string()), Value::Bool(true), Value::String("c".to_string())]);

        assert_eq!(<Vec<String>>::from_value(string_vec).unwrap(), vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert_eq!(
            <Vec<String>>::from_value(not_vec).unwrap_err().downcast::<PagesError>().unwrap(),
            PagesError::ValueParsing("expecting Value::Vec".to_string())
        );
        assert_eq!(
            <Vec<String>>::from_value(not_string_vec).unwrap_err().downcast::<PagesError>().unwrap(),
            PagesError::ValueParsing("expecting Value::String".to_string())
        );
    }
}

use reg_machine::machine::value::{TryFromValue, Value};

use super::list::{is_null_pair, list_ref, list_rest};

pub fn is_tagged_list(val: &Value, tag: &str) -> bool {
    if let Value::List(l) = val {
        !l.is_empty() && l[0].to_string() == tag
    } else {
        false
    }
}

pub fn is_self_evaluating(val: Value) -> bool {
    match val {
        Value::Num(_) | Value::String(_) => true,
        _ => false,
    }
}

pub fn is_variable(val: &Value) -> bool {
    if let Value::Symbol(_) = val {
        true
    } else {
        false
    }
}

pub fn is_application(val: Value) -> bool {
    if let Value::List(l) = val {
        !l.is_empty()
    } else {
        false
    }
}

pub fn is_compound_procedure(val: &Value) -> bool {
    is_tagged_list(val, "procedure")
}

pub fn is_boolean_true(val: Value) -> bool {
    match bool::try_from(val) {
        Ok(b) => b,
        _ => false,
    }
}

pub fn if_alternative(list: Value) -> Value {
    let cdddr = list_rest(&list, 3);
    if is_null_pair(&cdddr) {
        Value::Boolean(false)
    } else {
        list_ref(&cdddr, 0)
    }
}

pub fn definition_variable(list: Value) -> Value {
    let cadr = list_ref(&list, 1);
    if is_variable(&cadr) {
        cadr
    } else {
        list_ref(&cadr, 0)
    }
}

pub fn definition_value(list: Value) -> Value {
    let cadr = list_ref(&list, 1);
    if is_variable(&cadr) {
        list_ref(&list, 2)
    } else {
        let parameters = list_rest(&cadr, 1);
        let mut body = Vec::<Value>::try_from(list_rest(&list, 2)).unwrap();
        let mut result = vec![Value::new("lambda"), parameters];
        result.append(&mut body);
        Value::new(result)
    }
}

#[cfg(test)]
mod syntax_tests {
    use super::*;
    use reg_machine::{parser::rml_value, rmlvalue_to_value};

    fn parse(s: &str) -> Value {
        let (_, result) = rml_value(s).unwrap();
        rmlvalue_to_value(&result)
    }

    #[test]
    fn test_is_tagged_list() {
        assert!(is_tagged_list(&parse("(set! a b)"), "set!"));
        assert!(is_tagged_list(&parse("(define a b)"), "define"));
        assert!(is_tagged_list(
            &parse("(if (condition) consequent alternative)"),
            "if"
        ));
        assert!(is_tagged_list(&parse("(lambda (a b) c)"), "lambda"));
        assert!(is_tagged_list(
            &parse("(begin (set! a b) (display a) (newline))"),
            "begin"
        ));
    }

    #[test]
    fn test_is_self_evaluating() {
        assert!(is_self_evaluating(Value::new(1.2)));
        assert!(is_self_evaluating(Value::new(1)));
        assert!(is_self_evaluating(Value::new(r#""abcd""#)));
        assert!(!is_self_evaluating(Value::new(())));
        assert!(!is_self_evaluating(parse("(a b c)")));
    }

    #[test]
    fn test_is_variable() {
        assert!(is_variable(&Value::new("abcd")));
        assert!(!is_variable(&Value::new(1)));
    }

    #[test]
    fn test_is_application() {
        assert!(is_application(parse("(a b c d)")));
        assert!(!is_application(Value::new(())));
        assert!(!is_application(Value::new("a")));
    }

    #[test]
    fn test_is_boolean_true() {
        assert!(is_boolean_true(Value::new(true)));
        assert!(!is_boolean_true(Value::new(false)));
        assert!(!is_boolean_true(Value::new(())));
        assert!(!is_boolean_true(Value::new(1)));
        assert!(!is_boolean_true(Value::new("a")));
    }

    #[test]
    fn test_if_alternative() {
        assert_eq!(
            Value::Symbol("alternative".into()),
            if_alternative(parse("(if (condition) consequent alternative)"))
        );
        assert_eq!(
            Value::Boolean(false),
            if_alternative(parse("(if (condition) consequent)"))
        );
    }

    #[test]
    fn test_definition_variable() {
        assert_eq!(
            Value::Symbol("test".into()),
            definition_variable(parse("(define (test a) b)"))
        );
        assert_eq!(
            Value::Symbol("a".into()),
            definition_variable(parse("(define a b)"))
        );
    }

    #[test]
    fn test_definition_value() {
        assert_eq!(
            Value::Symbol("value".into()),
            definition_value(parse("(define test value)"))
        );
        assert_eq!(
            Value::List(vec![
                Value::Symbol("lambda".into()),
                Value::List(vec![Value::Symbol("a".into())]),
                Value::Symbol("b".into()),
                Value::Symbol("c".into()),
            ]),
            definition_value(parse("(define (test a) b c)"))
        );
        assert_eq!(
            Value::List(vec![
                Value::Symbol("lambda".into()),
                Value::List(vec![Value::Symbol("a".into())]),
                Value::List(vec![Value::Symbol("b".into()), Value::Symbol("c".into())]),
            ]),
            definition_value(parse("(define (test a) (b c))"))
        );
    }
}

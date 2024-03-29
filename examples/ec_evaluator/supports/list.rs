use log::debug;
use reg_machine::machine::value::Value;

pub fn list_ref(val: &Value, index: usize) -> Value {
    let article = if index == 0 { "the first" } else { "an" };
    if let Value::List(l) = val {
        debug!("list_ref: {:?}[{}]", l, index);
        let list: Vec<Value> = l
            .iter()
            .map(|v| v.clone())
            .filter(|v| !v.is_nil())
            .collect();
        if list.len() < index + 1 {
            panic!(
                "The object (), passed as {} argument to {}car, is not the correct type.",
                article,
                if index == 0 { "" } else { "safe-" },
            );
        }
        list[index].clone()
    } else {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            val,
            article,
            if index == 0 { "" } else { "safe-" },
        );
    }
}

pub fn list_rest(val: &Value, start: usize) -> Value {
    let article = if start == 1 { "the first" } else { "an" };
    if let Value::List(l) = val {
        debug!("list_rest: {:?}[{}..]", l, start);
        let list: Vec<Value> = l
            .iter()
            .map(|v| v.clone())
            .filter(|v| !v.is_nil())
            .collect();
        if list.len() < start {
            panic!(
                "The object (), passed as {} argument to {}cdr, is not the correct type.",
                article,
                if start == 1 { "" } else { "safe-" },
            );
        }
        let rest = l[start..].to_vec();
        debug!("rest: {:?}", rest);
        if !rest.is_empty() {
            if l.len() != list.len() {
                return rest[0].clone();
            }
        }
        Value::new(rest)
    } else {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            val,
            article,
            if start == 1 { "" } else { "safe-" },
        )
    }
}

pub fn is_null_pair(list: &Value) -> bool {
    if let Value::List(l) = list {
        l.is_empty()
    } else {
        false
    }
}

/// Current item is the last one in the vector.
pub fn is_last_one(list: &Value) -> bool {
    if let Value::List(l) = list {
        if l.len() == 2 {
            l[1].is_nil()
        } else {
            l.len() == 1
        }
    } else {
        false
    }
}

pub fn adjoin_arg(val: &Value, argl: &Value) -> Value {
    match (val, argl) {
        (item, Value::List(list)) => {
            let mut v = list.clone();
            v.push(item.clone());
            Value::List(v)
        }
        _ => panic!("Unable to adjoin {} and {}.", val, argl),
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;
    use reg_machine::{parser::rml_value, rmlvalue_to_value};

    fn parse(s: &str) -> Value {
        let (_, result) = rml_value(s).unwrap();
        rmlvalue_to_value(&result)
    }

    #[test]
    fn test_list_ref() {
        let list = parse("(a b c d)");
        assert_eq!(Value::Symbol("a".into()), list_ref(&list, 0));
        assert_eq!(Value::Symbol("b".into()), list_ref(&list, 1));
        assert_eq!(Value::Symbol("c".into()), list_ref(&list, 2));
        assert_eq!(Value::Symbol("d".into()), list_ref(&list, 3));
    }

    #[test]
    fn test_list_rest() {
        let list = parse("(a b c d)");
        assert_eq!(
            Value::List(vec![
                Value::Symbol("b".into()),
                Value::Symbol("c".into()),
                Value::Symbol("d".into())
            ]),
            list_rest(&list, 1)
        );
        assert_eq!(
            Value::List(vec![Value::Symbol("c".into()), Value::Symbol("d".into())]),
            list_rest(&list, 2)
        );
        assert_eq!(
            Value::List(vec![Value::Symbol("d".into())]),
            list_rest(&list, 3)
        );
        assert_eq!(Value::new(vec![]), list_rest(&list, 4));
    }

    #[test]
    fn test_is_null_pair() {
        assert!(is_null_pair(&Value::new(vec![])));
        assert!(!is_null_pair(&Value::new(())));
        assert!(!is_null_pair(&Value::new("a")));
        assert!(!is_null_pair(&Value::new(vec![Value::new("a")])));
    }

    #[test]
    fn test_is_last_one() {
        assert!(is_last_one(parse("(a)")));
        assert!(!is_last_one(parse("(a b)")));
    }

    #[test]
    fn test_adjoin_arg() {
        // `(adjoin-arg 'c '(a b)) => (a b c)`
        assert_eq!(parse("(a b c)"), adjoin_arg(parse("c"), parse("(a b)")));
        // `(adjoin-arg '(c d) '(a b)) => (a b (c d))`
        assert_eq!(
            parse("(a b (c d))"),
            adjoin_arg(parse("(c d)"), parse("(a b)"))
        );
        // `(adjoin-arg '(c d) '((a b))) => ((a b) (c d))`
        assert_eq!(
            parse("((a b) (c d))"),
            adjoin_arg(parse("(c d)"), parse("((a b))"))
        );
    }
}

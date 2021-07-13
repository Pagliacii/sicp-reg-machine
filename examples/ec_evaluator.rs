use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Div, Mul, Sub};

use fancy_regex::Regex;
use lazy_static::lazy_static;
use nom::combinator::map;

use reg_machine::{
    machine::{
        operation::Operation,
        value::{ToNumValue, Value},
        Operations,
    },
    make_machine,
    parser::{rml_float, rml_list, rml_number, rml_string, rml_symbol, RMLValue},
    rmlvalue_to_value,
};

/// Read from Stdin and replace `'` to `quote`.
/// Supports multiple lines.
fn read() -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"'(\([^'\)]*\)|\w)+(?!')").unwrap();
    }
    let mut balance = 0;
    let mut result = String::new();
    let mut previous = 0u8 as char;

    // Read multiple lines and balance parentheses.
    for b in io::stdin().bytes() {
        let mut c = b.unwrap() as char;
        if c == '(' {
            balance += 1;
        } else if c == ')' {
            balance -= 1;
        } else if c == '\n' {
            if balance == 0 {
                break;
            } else {
                c = ' ';
            }
        } else if c == ' ' && c == previous {
            continue;
        }
        previous = c;
        result.push(c);
    }

    String::from(RE.replace_all(&result, "(quote $1)"))
}

fn prompt_for_input(s: String) {
    println!("\n\n{}", s);
}

fn announce_output(s: String) {
    println!("\n{}", s);
}

fn user_print(s: String) {
    if is_compound_procedure(s.clone()) {
        println!(
            "(compound-procedure {} {} <procedure-env>)",
            CADR(s.clone()),
            CADDR(s)
        );
    } else {
        println!("{}", s);
    }
}

fn is_tagged_list(exp: &str, tag: &str) -> bool {
    if let Ok((_, node)) = rml_list(exp) {
        if let RMLValue::List(l) = node {
            if l.is_empty() {
                return false;
            }
            if let RMLValue::Symbol(s) = &l[0] {
                return s == tag;
            }
        }
    }
    false
}

fn is_self_evaluating(exp: String) -> bool {
    rml_float(&exp).is_ok() || rml_number(&exp).is_ok() || rml_string(&exp).is_ok()
}

fn is_variable(exp: String) -> bool {
    rml_symbol(&exp).is_ok()
}

fn is_quoted(exp: String) -> bool {
    is_tagged_list(&exp, "quote")
}

fn is_application(exp: String) -> bool {
    if let Ok((_, node)) = rml_list(&exp) {
        if let RMLValue::List(l) = node {
            return !l.is_empty();
        }
    }
    false
}

fn list_ref(list_exp: String, index: usize) -> Value {
    let article = if index == 0 { "the first" } else { "an" };
    let (_, result) = map(rml_list, |v| {
        if let RMLValue::List(l) = v {
            if l.len() < index + 1 {
                panic!(
                    "The object (), passed as {} argument to {}car, is not the correct type.",
                    article,
                    if index == 0 { "" } else { "safe-" },
                );
            }
            rmlvalue_to_value(&l[index])
        } else {
            unreachable!()
        }
    })(&list_exp)
    .unwrap_or_else(|_| {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            list_exp,
            article,
            if index == 0 { "" } else { "safe-" },
        )
    });
    result
}

fn list_rest(list_exp: String, start: usize) -> Value {
    let article = if start == 1 { "the first" } else { "an" };
    let (_, result) = map(rml_list, |v| {
        if let RMLValue::List(l) = v {
            if l.len() < start {
                panic!(
                    "The object (), passed as {} argument to {}cdr, is not the correct type.",
                    article,
                    if start == 1 { "" } else { "safe-" },
                );
            }
            Value::List(l[start..].iter().map(rmlvalue_to_value).collect())
        } else {
            unreachable!()
        }
    })(&list_exp)
    .unwrap_or_else(|_| {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            list_exp,
            article,
            if start == 1 { "" } else { "safe-" },
        )
    });
    result
}

/// Same behavior likes the same name procedure in Scheme.
const CAR: fn(String) -> Value = |exp: String| list_ref(exp, 0);
const CADR: fn(String) -> Value = |exp: String| list_ref(exp, 1);
const CADDR: fn(String) -> Value = |exp: String| list_ref(exp, 2);
const CADDDR: fn(String) -> Value = |exp: String| list_ref(exp, 3);
const CDR: fn(String) -> Value = |exp: String| list_rest(exp, 1);
const CDDR: fn(String) -> Value = |exp: String| list_rest(exp, 2);
const CDDDR: fn(String) -> Value = |exp: String| list_rest(exp, 3);

fn is_null_pair(exp: String) -> bool {
    let (_, result) = map(rml_list, |v| {
        if let RMLValue::List(l) = v {
            l.is_empty()
        } else {
            false
        }
    })(&exp)
    .unwrap_or(("", false));
    result
}

/// Current item is the last one in the vector.
fn is_last_one(exp: String) -> bool {
    if let Value::List(l) = CDR(exp) {
        return l.is_empty();
    }
    false
}

fn adjoin_arg(val: Value, argl: Value) -> Value {
    match (&val, &argl) {
        (Value::List(l1), Value::List(l2)) => {
            let mut v = l2.clone();
            v.extend(l1.clone());
            Value::List(v)
        }
        (other, Value::List(l)) => {
            let mut v = l.clone();
            v.push(other.clone());
            Value::List(v)
        }
        _ => panic!("Unable to adjoin {} and {}.", val, argl),
    }
}

fn is_compound_procedure(exp: String) -> bool {
    is_tagged_list(&exp, "procedure")
}

fn extend_environment(vars: Value, vals: Value, base_env: Value) -> Value {
    let variables: &Vec<Value>;
    let values: &Vec<Value>;
    let mut environment: HashMap<String, Value>;
    if let Value::List(l) = &vars {
        variables = l;
    } else {
        panic!(
            "Failed to extend the environment because `vars` {} isn't the Value::List",
            vars
        );
    }
    if let Value::List(l) = &vals {
        values = l;
    } else {
        panic!(
            "Failed to extend the environment because `vals` {} isn't the Value::List",
            vals
        );
    }
    if let Value::Map(m) = base_env {
        environment = m;
    } else {
        panic!(
            "Failed to extend the environment because `base_env` {} isn't the Value::Map",
            base_env
        );
    }
    if variables.len() < values.len() {
        panic!(
            "Failed to extend the environment because too many arguments supplied: `vars` = {}, `vals` = {}.",
            vars, vals
        );
    } else if variables.len() > values.len() {
        panic!(
            "Failed to extend the environment because too few arguments supplied: `vars` = {}, `vals` = {}.",
            vars, vals
        );
    }
    environment.extend(
        variables
            .iter()
            .zip(values.iter())
            .map(|(var, val)| (var.to_string(), val.clone())),
    );
    Value::Map(environment)
}

fn setup_environment() -> Value {
    lazy_static! {
        static ref PRIMITIVE_PROCEDURES: Operations = primitive_procedures();
    }
    let mut environment: HashMap<String, Value> = HashMap::new();
    environment.extend(
        PRIMITIVE_PROCEDURES
            .iter()
            .map(|(k, v)| (k.to_string(), Value::Op(v.clone()))),
    );
    environment.insert("true".into(), Value::Boolean(true));
    environment.insert("false".into(), Value::Boolean(false));
    Value::Map(environment)
}

fn get_global_environment() -> Value {
    setup_environment()
}

fn lookup_variable_value(var: String, env: Value) -> Value {
    if let Value::Map(actual_env) = env {
        match actual_env.get(&var) {
            Some(v) => return v.clone(),
            None => panic!("Unbound variable {}", var),
        }
    } else {
        panic!("Expected a Value::Map, got {}", env);
    }
}

fn set_variable_value(var: String, val: Value, env: Value) -> Value {
    if let Value::Map(mut actual_env) = env {
        actual_env.insert(var, val);
        Value::Map(actual_env)
    } else {
        env
    }
}

fn apply_primitive_procedure(proc: Value, argl: Value) -> Value {
    if let Value::Op(op) = &proc {
        if let Value::List(args) = &argl {
            if let Ok(v) = &op.perform(args.clone()) {
                v.clone()
            } else {
                panic!("Failed to apply procedure {} with arguments {}", proc, argl);
            }
        } else {
            panic!(
                "Failed to apply procedure {} with the argument {}",
                proc, argl
            );
        }
    } else {
        panic!("The `proc` argument isn't a applicable procedure: {}", proc);
    }
}

fn is_boolean_true(exp: String) -> bool {
    if let Ok((_, symbol)) = rml_symbol(&exp) {
        if let RMLValue::Symbol(s) = symbol {
            return s == "true";
        }
    }
    false
}

fn if_alternative(exp: String) -> Value {
    if let Value::List(l) = CDDDR(exp.clone()) {
        if !l.is_empty() {
            return CADDDR(exp);
        }
    }
    Value::Boolean(false)
}

fn definition_variable(exp: String) -> Value {
    match CADR(exp) {
        Value::String(s) => Value::String(s),
        Value::List(l) => {
            if l.len() == 0 {
                panic!(
                    "The object (), passed as an argument to safe-cdr, is not the correct type."
                );
            }
            return l[0].clone();
        }
        other => panic!(
            "The object {}, passed as an argument to safe-cdr, is not the correct type.",
            other
        ),
    }
}

fn definition_value(exp: String) -> Value {
    let cdr = match CDR(exp.clone()) {
        Value::List(l) => {
            if l.len() == 0 {
                panic!(
                    "The object (), passed as an argument to safe-cdr, is not the correct type."
                );
            }
            l.clone()
        }
        other => panic!(
            "The object {}, passed as an argument to safe-cdr, is not the correct type.",
            other
        ),
    };
    match &cdr[0] {
        Value::String(_) => CADDR(exp),
        Value::List(l) => {
            if l.len() == 0 {
                panic!(
                    "The object (), passed as an argument to safe-cdr, is not the correct type."
                );
            }
            let mut v: Vec<Value> = cdr[1..].to_vec();
            v.insert(0, Value::List(l[1..].to_vec()));
            v.insert(0, Value::String("lambda".into()));
            Value::List(v)
        }
        other => panic!(
            "The object {}, passed as an argument to safe-cdr, is not the correct type.",
            other
        ),
    }
}

fn apply<T>(op: &'static str, left: T, right: T) -> Value
where
    T: ToNumValue
        + fmt::Display
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + PartialEq
        + PartialOrd,
{
    match op {
        "+" => (left + right).to_value(),
        "-" => (left - right).to_value(),
        "*" => (left * right).to_value(),
        "/" => (left / right).to_value(),
        "=" => Value::Boolean(left == right),
        "<" => Value::Boolean(left < right),
        ">" => Value::Boolean(left > right),
        "<=" => Value::Boolean(left <= right),
        ">=" => Value::Boolean(left >= right),
        _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
    }
}

fn calculate(op: &'static str, left: Value, right: Value) -> Value {
    match &left {
        Value::Integer(l) => match &right {
            Value::Integer(r) => apply(op, *l, *r),
            Value::Float(r) => apply(op, *l as f64, *r),
            Value::BigNum(r) => apply(op, *l, *r as i32),
            Value::Pointer(r) => apply(op, *l, *r as i32),
            _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
        },
        Value::Float(l) => match &right {
            Value::Float(r) => apply(op, *l, *r),
            Value::Integer(r) => apply(op, *l, *r as f64),
            Value::BigNum(r) => apply(op, *l, *r as f64),
            Value::Pointer(r) => apply(op, *l, *r as f64),
            _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
        },
        Value::BigNum(l) => match &right {
            Value::BigNum(r) => apply(op, *l, *r),
            Value::Float(r) => apply(op, *l as f64, *r),
            Value::Integer(r) => apply(op, *l, *r as u64),
            Value::Pointer(r) => apply(op, *l, *r as u64),
            _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
        },
        Value::Pointer(l) => match &right {
            Value::Pointer(r) => apply(op, *l, *r),
            Value::BigNum(r) => apply(op, *l, *r as usize),
            Value::Float(r) => apply(op, *l as f64, *r),
            Value::Integer(r) => apply(op, *l, *r as usize),
            _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
        },
        _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
    }
}

fn primitive_procedures() -> Operations {
    let mut procedures: Operations = HashMap::new();
    procedures.insert("car", Operation::new(CAR));
    procedures.insert("cdr", Operation::new(CDR));
    procedures.insert("null?", Operation::new(is_null_pair));
    procedures.insert(
        "cons",
        Operation::new(|head: Value, tail: Value| Value::List(vec![head, tail])),
    );
    procedures.insert(
        "+",
        Operation::new(|left: Value, right: Value| calculate("+", left, right)),
    );
    procedures.insert(
        "-",
        Operation::new(|left: Value, right: Value| calculate("-", left, right)),
    );
    procedures.insert(
        "*",
        Operation::new(|left: Value, right: Value| calculate("*", left, right)),
    );
    procedures.insert(
        "/",
        Operation::new(|left: Value, right: Value| calculate("/", left, right)),
    );
    procedures.insert(
        "=",
        Operation::new(|left: Value, right: Value| calculate("=", left, right)),
    );
    procedures.insert(
        "<",
        Operation::new(|left: Value, right: Value| calculate("<", left, right)),
    );
    procedures.insert(
        ">",
        Operation::new(|left: Value, right: Value| calculate(">", left, right)),
    );
    procedures.insert(
        "<=",
        Operation::new(|left: Value, right: Value| calculate("<=", left, right)),
    );
    procedures.insert(
        ">=",
        Operation::new(|left: Value, right: Value| calculate(">=", left, right)),
    );
    procedures.insert("exit", Operation::new(|| std::process::exit(0)));
    procedures
}

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("read", Operation::new(read));
    operations.insert("prompt-for-input", Operation::new(prompt_for_input));
    operations.insert("announce-output", Operation::new(announce_output));
    operations.insert("user-print", Operation::new(user_print));
    operations.insert(
        "get-global-environment",
        Operation::new(get_global_environment),
    );
    operations.insert(
        "lookup-variable-value",
        Operation::new(lookup_variable_value),
    );
    operations.insert("set-variable-value", Operation::new(set_variable_value));
    operations.insert("extend-environment", Operation::new(extend_environment));
    operations.insert("self-evaluating?", Operation::new(is_self_evaluating));
    operations.insert("variable?", Operation::new(is_variable));
    operations.insert("quoted?", Operation::new(is_quoted));
    operations.insert("application?", Operation::new(is_application));
    operations.insert(
        "assignment?",
        Operation::new(|exp: String| is_tagged_list(&exp, "set!")),
    );
    operations.insert(
        "definition?",
        Operation::new(|exp: String| is_tagged_list(&exp, "define")),
    );
    operations.insert(
        "if?",
        Operation::new(|exp: String| is_tagged_list(&exp, "if")),
    );
    operations.insert(
        "lambda?",
        Operation::new(|exp: String| is_tagged_list(&exp, "lambda")),
    );
    operations.insert(
        "begin?",
        Operation::new(|exp: String| is_tagged_list(&exp, "begin")),
    );
    operations.insert("text-of-quotation", Operation::new(CADR));
    operations.insert("lambda-parameters", Operation::new(CADR));
    operations.insert("lambda-body", Operation::new(CDDR));
    operations.insert(
        "make-procedure",
        Operation::new(|unev: Value, exp: Value, env: Value| {
            vec![Value::String("procedure".into()), unev, exp, env]
        }),
    );
    operations.insert("operator", Operation::new(CAR));
    operations.insert("operands", Operation::new(CDR));
    operations.insert(
        "empty-arglist",
        Operation::new(|| Value::List(Vec::<Value>::new())),
    );
    operations.insert("no-operands?", Operation::new(is_null_pair));
    operations.insert("first-operand", Operation::new(CAR));
    operations.insert("last-operand?", Operation::new(is_last_one));
    operations.insert("adjoin-arg", Operation::new(adjoin_arg));
    operations.insert(
        "primitive-procedure?",
        Operation::new(|exp: String| is_tagged_list(&exp, "primitive")),
    );
    operations.insert("compound-procedure?", Operation::new(is_compound_procedure));
    operations.insert(
        "apply-primitive-procedure",
        Operation::new(apply_primitive_procedure),
    );
    operations.insert("procedure-parameters", Operation::new(CADR));
    operations.insert("procedure-body", Operation::new(CADDR));
    operations.insert("procedure-environment", Operation::new(CADDDR));
    operations.insert("begin-actions", Operation::new(CDR));
    operations.insert("first-exp", Operation::new(CAR));
    operations.insert("last-exp?", Operation::new(is_last_one));
    operations.insert("rest-exps", Operation::new(CDR));
    operations.insert("if-predicate", Operation::new(CADR));
    operations.insert("true?", Operation::new(is_boolean_true));
    operations.insert("if-alternative", Operation::new(if_alternative));
    operations.insert("if-consequent", Operation::new(CADDR));
    operations.insert("assignment-variable", Operation::new(CADR));
    operations.insert("assignment-value", Operation::new(CADDR));
    operations.insert("definition-variable", Operation::new(definition_variable));
    operations.insert("definition-value", Operation::new(definition_value));
    operations
}

fn main() {
    let controller_text: String =
        fs::read_to_string("examples/ec-eval.rkt").expect("Couldn't read from file");
    let register_names = vec![
        // `exp` is used to hold the expression to be evaluated
        "exp",
        // `env` contains the environment in which the evaluation is to be performed
        "env",
        // At the end of an evaluation, `val` contains the value obtained by
        // evaluating the expression in the designated environment
        "val",
        // The `continue` register is used to implement recursion,
        // as explained in Section 5.1.4.
        "continue",
        // The registers `proc`, `argl`, and `unev` are used in evaluating combinations.
        "proc", "argl", "unev",
    ];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &controller_text).unwrap();
    assert_eq!(Ok("Done"), machine.start());
}

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    #[test]
    fn test_is_self_evaluating() {
        assert!(is_self_evaluating("1.2".into()));
        assert!(is_self_evaluating("1".into()));
        assert!(is_self_evaluating(r#""abcd""#.into()));
        assert!(!is_self_evaluating("()".into()));
        assert!(!is_self_evaluating("(a b c)".into()));
    }

    #[test]
    fn test_is_variable() {
        assert!(is_variable("abcd".into()));
        assert!(!is_variable("1".into()));
    }

    #[test]
    fn test_is_quoted() {
        assert!(is_quoted("(quote (1 2 3 4))".into()));
        assert!(is_quoted("(quote a)".into()));
        assert!(!is_quoted("a".into()));
    }

    #[test]
    fn test_is_application() {
        assert!(is_application("(a b c d)".into()));
        assert!(!is_application("()".into()));
        assert!(!is_application("a".into()));
    }

    #[test]
    fn test_list_ref() {
        assert_eq!(Value::String("a".into()), list_ref("(a b c d)".into(), 0));
        assert_eq!(Value::String("b".into()), list_ref("(a b c d)".into(), 1));
        assert_eq!(Value::String("c".into()), list_ref("(a b c d)".into(), 2));
        assert_eq!(Value::String("d".into()), list_ref("(a b c d)".into(), 3));
    }

    #[test]
    fn test_list_rest() {
        assert_eq!(
            Value::List(vec![
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into())
            ]),
            list_rest("(a b c d)".into(), 1)
        );
        assert_eq!(
            Value::List(vec![Value::String("c".into()), Value::String("d".into())]),
            list_rest("(a b c d)".into(), 2)
        );
        assert_eq!(
            Value::List(vec![Value::String("d".into())]),
            list_rest("(a b c d)".into(), 3)
        );
        assert_eq!(
            Value::List(Vec::<Value>::new()),
            list_rest("(a b c d)".into(), 4)
        );
    }

    #[test]
    fn test_is_null_pair() {
        assert!(is_null_pair("()".into()));
        assert!(!is_null_pair("(a)".into()));
        assert!(!is_null_pair("a".into()));
    }

    #[test]
    fn test_is_last_one() {
        assert!(is_last_one("(a)".into()));
        assert!(!is_last_one("(a b)".into()));
    }

    #[test]
    fn test_adjoin_arg() {
        assert_eq!(
            Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
            ]),
            adjoin_arg(
                Value::String("c".into()),
                Value::List(vec![Value::String("a".into()), Value::String("b".into()),])
            )
        );
        assert_eq!(
            Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
            ]),
            adjoin_arg(
                Value::List(vec![Value::String("c".into()), Value::String("d".into()),]),
                Value::List(vec![Value::String("a".into()), Value::String("b".into()),]),
            )
        );
    }

    #[test]
    fn test_is_compound_procedure() {
        assert!(is_compound_procedure("(procedure)".into()));
        assert!(is_compound_procedure("(procedure a b c d)".into()));
        assert!(!is_compound_procedure("(a b c d)".into()));
        assert!(!is_compound_procedure("a".into()));
    }

    #[test]
    fn test_is_boolean_true() {
        assert!(is_boolean_true("true".into()));
        assert!(!is_boolean_true("false".into()));
        assert!(!is_boolean_true("()".into()));
        assert!(!is_boolean_true("1".into()));
        assert!(!is_boolean_true("a".into()));
    }

    #[test]
    fn test_if_alternative() {
        assert_eq!(
            Value::String("alternative".into()),
            if_alternative("(if (condition) consequent alternative)".into())
        );
        assert_eq!(
            Value::Boolean(false),
            if_alternative("(if (condition) consequent)".into())
        );
    }

    #[test]
    fn test_definition_variable() {
        assert_eq!(
            Value::String("test".into()),
            definition_variable("(define (test a) b)".into())
        );
    }

    #[test]
    fn test_definition_value() {
        assert_eq!(
            Value::String("value".into()),
            definition_value("(define test value)".into())
        );
        assert_eq!(
            Value::List(vec![
                Value::String("lambda".into()),
                Value::List(vec![Value::String("a".into())]),
                Value::String("b".into()),
                Value::String("c".into()),
            ]),
            definition_value("(define (test a) b c)".into())
        );
        assert_eq!(
            Value::List(vec![
                Value::String("lambda".into()),
                Value::List(vec![Value::String("a".into())]),
                Value::List(vec![Value::String("b".into()), Value::String("c".into()),]),
            ]),
            definition_value("(define (test a) (b c))".into())
        );
    }

    #[test]
    fn test_setup_environment() {
        if let Value::Map(env) = setup_environment() {
            assert_eq!(Some(&Value::Boolean(true)), env.get("true"));
            assert_eq!(Some(&Value::Boolean(false)), env.get("false"));
            assert!(env.contains_key("cons"));
            assert!(env.contains_key("exit"));
        } else {
            panic!("The function setup_environment doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_extend_environment() {
        let vars = Value::new(vec![Value::new("a"), Value::new("b"), Value::new("c")]);
        let vals = Value::new(vec![Value::new(1), Value::new(1.0), Value::new(1u64)]);
        if let Value::Map(env) = extend_environment(vars, vals, setup_environment()) {
            assert_eq!(Some(&Value::Integer(1)), env.get("a"));
            assert_eq!(Some(&Value::Float(1.0)), env.get("b"));
            assert_eq!(Some(&Value::BigNum(1)), env.get("c"));
        } else {
            panic!("The function extend_environment doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_set_variable_value() {
        if let Value::Map(env) =
            set_variable_value("a".into(), Value::new(1), get_global_environment())
        {
            assert_eq!(Some(&Value::Integer(1)), env.get("a"));
        } else {
            panic!("The function set_variable_value doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_apply_primitive_procedure() {
        let procedures = primitive_procedures();
        let proc = Value::new(procedures.get("+").unwrap().clone());
        let res = apply_primitive_procedure(proc, Value::new(vec![Value::new(1), Value::new(1)]));
        assert_eq!(Value::Integer(2), res);
    }
}

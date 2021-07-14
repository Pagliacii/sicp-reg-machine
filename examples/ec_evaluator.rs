use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::ops::{Add, Div, Mul, Sub};

use fancy_regex::Regex;
use lazy_static::lazy_static;

use reg_machine::{
    machine::{
        operation::Operation,
        value::{ToNumValue, TryFromValue, Value},
        Operations,
    },
    make_machine,
    parser::rml_value,
    rmlvalue_to_value,
};

/// Read from Stdin and replace `'` to `quote`.
/// Supports multiple lines.
fn read() -> Value {
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

    let (_, res) = rml_value(&RE.replace_all(&result, "(quote $1)")).unwrap();
    rmlvalue_to_value(&res)
}

fn display(val: Value) {
    let s = match val {
        Value::String(v) => v,
        others => others.to_string(),
    };
    print!("{}", s);
}

fn prompt_for_input(val: Value) {
    println!();
    println!();
    display(val);
    println!();
}

fn announce_output(val: Value) {
    println!();
    display(val);
    println!();
}

fn user_print(s: Value) {
    if is_compound_procedure(&s) {
        println!(
            "(compound-procedure {} {} <procedure-env>)",
            list_ref(&s, 1),
            list_ref(&s, 2),
        );
    } else {
        println!("{}", s);
    }
}

fn is_tagged_list(val: &Value, tag: &str) -> bool {
    if let Value::List(l) = val {
        !l.is_empty() && l[0].to_string() == tag
    } else {
        false
    }
}

fn is_self_evaluating(val: Value) -> bool {
    match val {
        Value::Num(_) | Value::String(_) => true,
        _ => false,
    }
}

fn is_variable(val: &Value) -> bool {
    if let Value::Symbol(_) = val {
        true
    } else {
        false
    }
}

fn is_application(val: Value) -> bool {
    if let Value::List(l) = val {
        !l.is_empty()
    } else {
        false
    }
}

fn list_ref(val: &Value, index: usize) -> Value {
    let article = if index == 0 { "the first" } else { "an" };
    if let Value::List(l) = val {
        if l.len() < index + 1 {
            panic!(
                "The object (), passed as {} argument to {}car, is not the correct type.",
                article,
                if index == 0 { "" } else { "safe-" },
            );
        }
        l[index].clone()
    } else {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            val,
            article,
            if index == 0 { "" } else { "safe-" },
        );
    }
}

fn list_rest(val: &Value, start: usize) -> Value {
    let article = if start == 1 { "the first" } else { "an" };
    if let Value::List(l) = val {
        if l.len() < start {
            panic!(
                "The object (), passed as {} argument to {}cdr, is not the correct type.",
                article,
                if start == 1 { "" } else { "safe-" },
            );
        }
        Value::new(l[start..].to_vec())
    } else {
        panic!(
            "The object {}, passed as {} argument to {}cdr, is not the correct type.",
            val,
            article,
            if start == 1 { "" } else { "safe-" },
        )
    }
}

/// Same behavior likes the same name procedure in Scheme.
const CAR: fn(Value) -> Value = |exp: Value| list_ref(&exp, 0);
const CADR: fn(Value) -> Value = |exp: Value| list_ref(&exp, 1);
const CADDR: fn(Value) -> Value = |exp: Value| list_ref(&exp, 2);
const CADDDR: fn(Value) -> Value = |exp: Value| list_ref(&exp, 3);
const CDR: fn(Value) -> Value = |exp: Value| list_rest(&exp, 1);
const CDDR: fn(Value) -> Value = |exp: Value| list_rest(&exp, 2);

fn is_null_pair(list: &Value) -> bool {
    if let Value::List(l) = list {
        l.is_empty()
    } else {
        false
    }
}

/// Current item is the last one in the vector.
fn is_last_one(list: Value) -> bool {
    if let Value::List(l) = list {
        l.len() == 1
    } else {
        false
    }
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

fn is_compound_procedure(val: &Value) -> bool {
    is_tagged_list(val, "procedure")
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

lazy_static! {
    static ref PRIMITIVE_PROCEDURES: Operations = primitive_procedures();
}

fn setup_environment() -> Value {
    let mut environment: HashMap<String, Value> = HashMap::new();
    environment.extend(PRIMITIVE_PROCEDURES.iter().map(|(k, v)| {
        (
            k.to_string(),
            Value::new(vec![Value::new("primitive"), Value::Op(v.clone())]),
        )
    }));
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
        match actual_env.get_mut(&var) {
            Some(v) => {
                *v = val;
                Value::Map(actual_env)
            }
            _ => panic!("Unbound variable: SET! {}", var),
        }
    } else {
        env
    }
}

fn define_variable(var: String, val: Value, env: Value) -> Value {
    if let Value::Map(mut actual_env) = env {
        actual_env.insert(var, val);
        Value::Map(actual_env)
    } else {
        env
    }
}

fn apply_primitive_procedure(proc: Value, argl: Value) -> Value {
    if let Value::List(pair) = &proc {
        if pair.len() < 2 || Value::new("primitive") != pair[0] {
            panic!("This `proc` {} isn't a primitive procedure", proc);
        }
        let op = match &pair[1] {
            Value::Op(o) => o.clone(),
            _ => panic!("This `proc` {} isn't a primitive procedure", proc),
        };
        if let Value::List(args) = &argl {
            op.perform(args.clone()).unwrap()
        } else {
            panic!(
                "Failed to apply a primitive procedure with the argument {}",
                argl
            );
        }
    } else {
        panic!("The `proc` argument isn't a applicable procedure: {}", proc);
    }
}

fn is_boolean_true(val: Value) -> bool {
    match bool::try_from(val) {
        Ok(b) => b,
        _ => false,
    }
}

fn if_alternative(list: Value) -> Value {
    let cdddr = list_rest(&list, 3);
    if is_null_pair(&cdddr) {
        Value::Boolean(false)
    } else {
        list_ref(&cdddr, 0)
    }
}

fn definition_variable(list: Value) -> Value {
    let cadr = list_ref(&list, 1);
    if is_variable(&cadr) {
        cadr
    } else {
        list_ref(&cadr, 0)
    }
}

fn definition_value(list: Value) -> Value {
    let cadr = list_ref(&list, 1);
    if is_variable(&cadr) {
        list_ref(&list, 2)
    } else {
        Value::new(vec![
            Value::new("lambda"),
            list_rest(&cadr, 1),
            list_rest(&list, 2),
        ])
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

fn calculate(op: &'static str, left: &Value, right: &Value) -> Value {
    match (left, right) {
        (Value::Num(l), Value::Num(r)) => apply(op, *l, *r),
        _ => panic!("Unable to apply operation {} to {} and {}", op, left, right),
    }
}

fn primitive_procedures() -> Operations {
    let mut procedures: Operations = HashMap::new();
    procedures.insert("car", Operation::new(CAR));
    procedures.insert("cdr", Operation::new(CDR));
    procedures.insert("null?", Operation::new(|pair: Value| is_null_pair(&pair)));
    procedures.insert(
        "cons",
        Operation::new(|head: Value, tail: Value| Value::List(vec![head, tail])),
    );
    procedures.insert(
        "+",
        Operation::new(|left: Value, right: Value| calculate("+", &left, &right)),
    );
    procedures.insert(
        "-",
        Operation::new(|left: Value, right: Value| calculate("-", &left, &right)),
    );
    procedures.insert(
        "*",
        Operation::new(|left: Value, right: Value| calculate("*", &left, &right)),
    );
    procedures.insert(
        "/",
        Operation::new(|left: Value, right: Value| calculate("/", &left, &right)),
    );
    procedures.insert(
        "=",
        Operation::new(|left: Value, right: Value| calculate("=", &left, &right)),
    );
    procedures.insert(
        "<",
        Operation::new(|left: Value, right: Value| calculate("<", &left, &right)),
    );
    procedures.insert(
        ">",
        Operation::new(|left: Value, right: Value| calculate(">", &left, &right)),
    );
    procedures.insert(
        "<=",
        Operation::new(|left: Value, right: Value| calculate("<=", &left, &right)),
    );
    procedures.insert(
        ">=",
        Operation::new(|left: Value, right: Value| calculate(">=", &left, &right)),
    );
    procedures.insert("exit", Operation::new(|| std::process::exit(0)));
    procedures
}

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("read", Operation::new(read));
    operations.insert("debug", Operation::new(|v: Value| println!("{:?}", v)));
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
    operations.insert("define-variable", Operation::new(define_variable));
    operations.insert("self-evaluating?", Operation::new(is_self_evaluating));
    operations.insert("variable?", Operation::new(|v: Value| is_variable(&v)));
    operations.insert(
        "quoted?",
        Operation::new(|v: Value| is_tagged_list(&v, "quote")),
    );
    operations.insert("application?", Operation::new(is_application));
    operations.insert(
        "assignment?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "set!")),
    );
    operations.insert(
        "definition?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "define")),
    );
    operations.insert(
        "if?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "if")),
    );
    operations.insert(
        "lambda?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "lambda")),
    );
    operations.insert(
        "begin?",
        Operation::new(|exp: Value| is_tagged_list(&exp, "begin")),
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
    operations.insert(
        "no-operands?",
        Operation::new(|pair: Value| is_null_pair(&pair)),
    );
    operations.insert("first-operand", Operation::new(CAR));
    operations.insert("last-operand?", Operation::new(is_last_one));
    operations.insert("rest-operands", Operation::new(CDR));
    operations.insert("adjoin-arg", Operation::new(adjoin_arg));
    operations.insert(
        "primitive-procedure?",
        Operation::new(|list: Value| Value::Symbol("primitive".into()) == list_ref(&list, 0)),
    );
    operations.insert(
        "compound-procedure?",
        Operation::new(|v: Value| is_compound_procedure(&v)),
    );
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
                Value::List(vec![Value::Symbol("b".into()), Value::Symbol("c".into())]),
            ]),
            definition_value(parse("(define (test a) b c)"))
        );
        assert_eq!(
            Value::List(vec![
                Value::Symbol("lambda".into()),
                Value::List(vec![Value::Symbol("a".into())]),
                Value::List(vec![Value::List(vec![
                    Value::Symbol("b".into()),
                    Value::Symbol("c".into())
                ])]),
            ]),
            definition_value(parse("(define (test a) (b c))"))
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
            assert_eq!(Some(&Value::Num(1.0)), env.get("a"));
            assert_eq!(Some(&Value::Num(1.0)), env.get("b"));
            assert_eq!(Some(&Value::Num(1.0)), env.get("c"));
        } else {
            panic!("The function extend_environment doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_define_variable() {
        if let Value::Map(env) =
            define_variable("a".into(), Value::new(1), get_global_environment())
        {
            assert_eq!(Some(&Value::Num(1.0)), env.get("a"));
        } else {
            panic!("The function define_variable doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_set_variable_value() {
        let env: HashMap<String, Value>;
        if let Value::Map(actual_env) =
            define_variable("a".into(), Value::new(1), get_global_environment())
        {
            assert_eq!(Some(&Value::Num(1.0)), actual_env.get("a"));
            env = actual_env;
        } else {
            panic!("The function define_variable doesn't return a Value::Map.")
        }
        if let Value::Map(actual_env) =
            set_variable_value("a".into(), Value::new(2), Value::new(env))
        {
            assert_eq!(Some(&Value::Num(2.0)), actual_env.get("a"));
        } else {
            panic!("The function set_variable_value doesn't return a Value::Map.")
        }
    }

    #[test]
    fn test_lookup_variable_value() {
        let env = get_global_environment();
        let env = define_variable("a".into(), Value::new(1), env);
        let val = lookup_variable_value("a".into(), env);
        assert_eq!(Value::new(1), val);
    }

    #[test]
    fn test_apply_primitive_procedure() {
        let env = get_global_environment();
        let proc = lookup_variable_value(Value::new("+").to_string(), env);
        let res = apply_primitive_procedure(proc, Value::new(vec![Value::new(1), Value::new(1)]));
        assert_eq!(Value::Num(2.0), res);
    }
}

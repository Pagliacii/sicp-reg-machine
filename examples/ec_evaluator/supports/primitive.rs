use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
};

use reg_machine::machine::{
    operation::Operation,
    value::{TryFromValue, Value},
    Operations,
};

use super::{
    io::display,
    list::{is_null_pair, CAR, CDR},
};

pub fn apply_primitive_procedure(proc: Vec<Value>, argl: Value) -> Value {
    let pair = &proc;
    if pair.len() < 2 || Value::new("primitive") != pair[0] {
        panic!(
            "Unable to apply this `proc` argument: {}.",
            Value::new(proc)
        );
    }
    let op = match &pair[1] {
        Value::Op(o) => o.clone(),
        other => panic!("The `{}` isn't a primitive procedure.", other),
    };
    if let Value::List(args) = &argl {
        op.perform(args.clone()).unwrap()
    } else {
        panic!(
            "Failed to apply a primitive procedure with the argument {}.",
            argl
        );
    }
}

fn accumulate<T>(items: Vec<f64>, init_val: f64, combiner: &T) -> f64
where
    T: Fn(f64, f64) -> f64,
{
    if items.is_empty() {
        init_val
    } else {
        combiner(
            items[0],
            accumulate(items[1..].to_vec(), init_val, combiner),
        )
    }
}

fn addition(items: Vec<f64>) -> f64 {
    accumulate(items, 0.0, &f64::add)
}

fn subtraction(mut items: Vec<f64>) -> f64 {
    if items.is_empty() {
        panic!("[SUBTRACTION] Requires at lease 1 item.");
    } else if items.len() == 1 {
        items.insert(0, 0.0);
    }
    accumulate(items, 0.0, &f64::sub)
}

fn multiplication(items: Vec<f64>) -> f64 {
    if items.contains(&0.0) {
        0.0
    } else {
        accumulate(items, 1.0, &f64::mul)
    }
}

fn division(mut items: Vec<f64>) -> f64 {
    if items[1..].contains(&0.0) {
        panic!("[DIVISION] Division by zero.");
    } else if items.len() == 1 {
        items.insert(0, 1.0);
    }
    accumulate(items, 0.0, &f64::div)
}

fn comparison<T>(items: Vec<f64>, comparator: T) -> bool
where
    T: Fn(&f64, &f64) -> bool,
{
    if items.len() < 2 {
        true
    } else {
        comparator(&items[0], &items[1]) && comparison(items[1..].to_vec(), comparator)
    }
}

fn do_arithmetic(args: Value, op: &'static str) -> Value {
    let items = Vec::<f64>::try_from(args).unwrap();
    match op {
        "+" => Value::new(addition(items)),
        "-" => Value::new(subtraction(items)),
        "*" => Value::new(multiplication(items)),
        "/" => Value::new(division(items)),
        "=" => Value::new(comparison(items, f64::eq)),
        "<" => Value::new(comparison(items, f64::lt)),
        "<=" => Value::new(comparison(items, f64::le)),
        ">" => Value::new(comparison(items, f64::gt)),
        ">=" => Value::new(comparison(items, f64::ge)),
        _ => panic!("Unsupported arithmetic operator: {}", op),
    }
}

pub fn primitive_procedures() -> Operations {
    let mut procedures: Operations = HashMap::new();
    procedures.insert("car", Operation::new(CAR));
    procedures.insert("cdr", Operation::new(CDR));
    procedures.insert("null?", Operation::new(|pair: Value| is_null_pair(&pair)));
    procedures.insert(
        "cons",
        Operation::new(|head: Value, tail: Value| {
            if let Value::List(mut l) = tail {
                l.insert(0, head);
                l.clone()
            } else {
                vec![head, tail]
            }
        }),
    );
    procedures.insert("+", Operation::new(|args: Value| do_arithmetic(args, "+")));
    procedures.insert("-", Operation::new(|args: Value| do_arithmetic(args, "-")));
    procedures.insert("*", Operation::new(|args: Value| do_arithmetic(args, "*")));
    procedures.insert("/", Operation::new(|args: Value| do_arithmetic(args, "/")));
    procedures.insert("=", Operation::new(|args: Value| do_arithmetic(args, "=")));
    procedures.insert("<", Operation::new(|args: Value| do_arithmetic(args, "<")));
    procedures.insert(">", Operation::new(|args: Value| do_arithmetic(args, ">")));
    procedures.insert(
        "<=",
        Operation::new(|args: Value| do_arithmetic(args, "<=")),
    );
    procedures.insert(
        ">=",
        Operation::new(|args: Value| do_arithmetic(args, ">=")),
    );
    procedures.insert("exit", Operation::new(|| std::process::exit(0)));
    procedures.insert("display", Operation::new(display));
    procedures.insert("newline", Operation::new(|| println!()));
    procedures
}

#[cfg(test)]
mod tests {
    use super::super::environment::{get_global_environment, manipulate_env};
    use super::*;
    use reg_machine::machine::value::TryFromValue;

    #[test]
    fn test_apply_primitive_procedure() {
        let env = usize::try_from(get_global_environment()).unwrap();
        let proc = manipulate_env("lookup", env, &vec![Value::new("+")]);
        let res = apply_primitive_procedure(
            Vec::<Value>::try_from(proc).unwrap(),
            Value::new(vec![Value::new(1), Value::new(1)]),
        );
        assert_eq!(Value::Num(2.0), res);
    }
}

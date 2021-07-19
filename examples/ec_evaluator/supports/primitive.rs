use std::{
    collections::HashMap,
    fmt,
    ops::{Add, Div, Mul, Sub},
};

use reg_machine::machine::{
    operation::Operation,
    value::{ToNumValue, Value},
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

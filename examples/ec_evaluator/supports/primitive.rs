use reg_machine::{
    machine::{
        procedure::Procedure,
        value::{ToValue, Value},
    },
    math,
};

use super::{
    io::display,
    list::{is_null_pair, list_ref, list_rest},
};

pub fn apply_primitive_procedure(proc: Vec<Value>, args: Vec<Value>) -> Value {
    let pair = &proc;
    if pair.len() < 2 || Value::new("primitive") != pair[0] {
        panic!(
            "Unable to apply this `proc` argument: {}.",
            Value::new(proc)
        );
    }
    if !pair[1].is_procedure() {
        panic!("The `{}` isn't a primitive procedure.", pair[1]);
    }
    pair[1].perform(args).unwrap()
}

pub fn primitive_procedures() -> Vec<Procedure> {
    let mut procedures: Vec<Procedure> = vec![];
    procedures.push(Procedure::new("car", 1, |args| list_ref(&args[0], 0)));
    procedures.push(Procedure::new("cdr", 1, |args| list_rest(&args[0], 1)));
    procedures.push(Procedure::new("cons", 2, |args| {
        let head = args[0].clone();
        let mut tail = args[1].clone();
        if let Value::List(l) = &mut tail {
            l.insert(0, head);
            tail
        } else {
            vec![head, tail].to_value()
        }
    }));
    procedures.push(Procedure::new("null?", 1, |args| is_null_pair(&args[0])));
    procedures.push(Procedure::new("+", 0, math::addition));
    procedures.push(Procedure::new("-", 1, math::subtraction));
    procedures.push(Procedure::new("*", 0, math::multiplication));
    procedures.push(Procedure::new("/", 1, math::division));
    procedures.push(Procedure::new("=", 0, math::equal));
    procedures.push(Procedure::new("<", 0, math::less_than));
    procedures.push(Procedure::new(">", 0, math::greater_than));
    procedures.push(Procedure::new("<=", 0, math::less_than_or_equal_to));
    procedures.push(Procedure::new(">=", 0, math::greater_than_or_equal_to));
    procedures.push(Procedure::new("exit", 0, |_| std::process::exit(0)));
    procedures.push(Procedure::new("display", 1, |args| display(&args[0])));
    procedures.push(Procedure::new("newline", 0, |_| println!()));
    // Support logical composition operations: `and`, `or` and `not`.
    procedures.push(Procedure::new("and", 0, |args| {
        for value in args.iter() {
            if value.is_false() {
                return false.to_value();
            }
        }
        args.last().map_or_else(|| true.to_value(), |v| v.clone())
    }));
    procedures.push(Procedure::new("or", 0, |args| {
        for value in args.iter() {
            if !value.is_bool() {
                return value.clone();
            }
            if value.is_true() {
                return true.to_value();
            }
        }
        false.to_value()
    }));
    procedures.push(Procedure::new("not", 1, |args| {
        if args.len() > 1 {
            panic!("The procedure #[not] has been called with {} arguments; it requires exactly 1 argument.", args.len());
        }
        args[0].is_false()
    }));
    procedures.push(Procedure::new("list", 0, |args| args.to_value()));
    procedures
}

#[cfg(test)]
mod primitive_tests {
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

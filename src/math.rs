use crate::machine::value::Value;

pub fn addition(items: Vec<Value>) -> Value {
    items.into_iter().fold(Value::zero(), |acc, x| acc + x)
}

pub fn subtraction(mut items: Vec<Value>) -> Value {
    if items.is_empty() {
        panic!("[SUBTRACTION] Requires at lease 1 item.");
    } else if items.len() == 1 {
        items.insert(0, Value::zero());
    }
    items[0].clone() - addition(items[1..].to_vec())
}

pub fn multiplication(items: Vec<Value>) -> Value {
    if items.contains(&Value::zero()) {
        Value::zero()
    } else {
        items.into_iter().fold(Value::one(), |acc, x| acc * x)
    }
}

pub fn division(mut items: Vec<Value>) -> Value {
    if items.is_empty() {
        panic!("[DIVISION] Requires at lease 1 item.");
    } else if items[1..].contains(&Value::zero()) {
        panic!("[DIVISION] Cannot divide by Value::Num(0.0).");
    } else if items[0].eq_num(0) {
        return Value::zero();
    } else if items.len() == 1 {
        items.insert(0, Value::one());
    }
    items[0].clone() / multiplication(items[1..].to_vec())
}

fn comparison<T>(items: Vec<Value>, comparator: T) -> bool
where
    T: Fn(&Value, &Value) -> bool,
{
    if items.len() < 2 {
        true
    } else {
        comparator(&items[0], &items[1]) && comparison(items[1..].to_vec(), comparator)
    }
}

pub fn equal(items: Vec<Value>) -> bool {
    comparison(items, Value::eq)
}

pub fn less_than(items: Vec<Value>) -> bool {
    comparison(items, Value::lt)
}

pub fn greater_than(items: Vec<Value>) -> bool {
    comparison(items, Value::gt)
}

pub fn less_than_or_equal_to(items: Vec<Value>) -> bool {
    comparison(items, Value::le)
}

pub fn greater_than_or_equal_to(items: Vec<Value>) -> bool {
    comparison(items, Value::ge)
}

#[cfg(test)]
mod math_tests {
    use super::*;
    use crate::machine::value::{ToValue, TryFromValue};

    #[test]
    fn test_addition() {
        let items = (1..=10).map(i32::to_value).collect();
        let sum = i32::try_from(&addition(items)).unwrap();
        assert_eq!((1..=10).fold(0, |acc, x| acc + x), sum);
        let sum = addition(Vec::<Value>::new());
        assert_eq!(Value::Num(0.0), sum);
    }

    #[test]
    fn test_subtraction() {
        let difference = subtraction(vec![(-1).to_value()]);
        assert_eq!(Value::Num(1.0), difference);
        let items = (1..=10).rev().map(i32::to_value).collect();
        let difference = i32::try_from(&subtraction(items)).unwrap();
        assert_eq!((1..10).rev().fold(10, |acc, x| acc - x), difference);
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(Value::Num(1.0), multiplication(Vec::<Value>::new()));
        let items = (1..=10).map(i32::to_value).collect();
        let production = i32::try_from(&multiplication(items)).unwrap();
        assert_eq!((1..=10).fold(1, |acc, x| acc * x), production);
    }

    #[test]
    fn test_division() {
        assert_eq!(Value::Num(0.5), division(vec![2.to_value()]));
        let items = (1..=10).rev().map(i32::to_value).collect();
        let expected = (1..10).map(|i| i as f64).rev().fold(10.0, |acc, x| acc / x);
        let quotient = f64::try_from(&division(items)).unwrap();
        let tolerance = (quotient - expected).abs();
        assert!(tolerance < 1e-20);
    }
}

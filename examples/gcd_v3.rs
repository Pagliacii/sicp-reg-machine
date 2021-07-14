use std::collections::HashMap;

use reg_machine::{
    machine::{
        operation::Operation,
        value::{TryFromValue, Value},
        Operations,
    },
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
 test-b
   (test (op =) (reg b) (const 0))
   (branch (label gcd-done))
   (assign t (reg a))
 rem-loop
   (test (op <) (reg t) (reg b))
   (branch (label rem-done))
   (assign t (op -) (reg t) (reg b))
   (goto (label rem-loop))
 rem-done
   (assign a (reg b))
   (assign b (reg t))
   (goto (label test-b))
 gcd-done)
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert("=", Operation::new(|a: i32, b: i32| a == b));
    operations.insert("<", Operation::new(|a: i32, b: i32| a < b));
    operations.insert("-", Operation::new(|a: i32, b: i32| a - b));
    operations
}

fn main() {
    let register_names = vec!["a", "b", "t"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    machine.set_register_content("a", Value::new(1023)).unwrap();
    machine.set_register_content("b", Value::new(27)).unwrap();
    assert_eq!(Ok("Done"), machine.start());
    let value = machine.get_register_content("a").unwrap();
    println!("gcd(1023, 27) = {}", i32::try_from(value).unwrap());
}

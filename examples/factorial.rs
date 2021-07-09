use std::collections::HashMap;

use reg_machine::{
    machine::{
        operation::Operation,
        value::{FromValue, Value},
        Operations,
    },
    make_machine,
};

const CONTROLLER_TEXT: &str = r#"
(controller
   (assign p (const 1))
   (assign c (const 1))
 test-c
   (test (op >) (reg c) (reg n))
   (branch (label factorial-done))
   (assign p (op *) (reg p) (reg c))
   (assign c (op +) (reg c) (const 1))
   (goto (label test-c))
 factorial-done)
"#;

fn operations() -> Operations {
    let mut operations: Operations = HashMap::new();
    operations.insert(">", Operation::new(|a: u64, b: u64| a > b));
    operations.insert("*", Operation::new(|a: u64, b: u64| a * b));
    operations.insert("+", Operation::new(|a: u64, b: u64| a + b));
    operations
}

fn main() {
    let register_names = vec!["n", "p", "c"];
    let operations = operations();
    let mut machine = make_machine(register_names, &operations, &CONTROLLER_TEXT).unwrap();
    machine
        .set_register_content("n", Value::Integer(16))
        .unwrap();
    assert_eq!(Ok("Done"), machine.start());
    let value = machine.get_register_content("p").unwrap();
    println!("factorial(16) = {}", u64::from_value(value).unwrap());
}
